use itertools::Itertools;
use std::sync::OnceLock;
use {super::*, updater::BlockData};

#[derive(Debug, PartialEq)]
pub(crate) enum Error {
  Recoverable { height: u32, depth: u32 },
  Unrecoverable,
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Recoverable { height, depth } => {
        write!(f, "{depth} block deep reorg detected at height {height}")
      }
      Self::Unrecoverable => write!(f, "unrecoverable reorg detected"),
    }
  }
}

impl std::error::Error for Error {}

const MAX_SAVEPOINTS: u32 = 6;
const SAVEPOINT_INTERVAL: u32 = 5;
const CHAIN_TIP_DISTANCE: u32 = 21;

pub(crate) struct Reorg {}

static POINTS: OnceLock<Mutex<BTreeMap<u64, u64>>> = OnceLock::new();
impl Reorg {
  pub fn insert_point(block_height: u64, point: u64) {
    let points = POINTS.get_or_init(Default::default);
    let mut points = points.lock().unwrap();
    points.insert(block_height, point);
  }

  pub fn remove_point(point: u64) {
    let points = POINTS.get_or_init(Default::default);
    let mut points = points.lock().unwrap();
    points.retain(|_, p| *p != point);
  }

  /// remote all the point that larger than the point
  pub fn retain_point(end_point: u64) -> Vec<u64> {
    let points = POINTS.get_or_init(Default::default);
    let mut points = points.lock().unwrap();
    let mut removed_points = vec![];
    for (h, p) in points.iter() {
      if *p > end_point {
        removed_points.push((*h, *p))
      }
    }
    for (h, _) in removed_points.iter() {
      points.remove(&h);
    }
    removed_points.into_iter().map(|(_h, p)| p).collect()
  }

  /// find the first point(from high to low) when the reg_block_height < block_height, for example the POINTS is:
  /// 100 -> 1, 110 -> 2, 120 -> 3, 130 -> 4, 140 -> 5
  /// now current block height is 145, and we found that we need to rollback to block height 125
  /// we need to find the 120 -> 3 pair
  pub fn query_point(reg_block_height: u64) -> Option<u64> {
    let points = POINTS.get_or_init(Default::default);
    let points = points.lock().unwrap();
    // range from height block to low block
    for (block_height, point) in points.iter().sorted_by(|a, b| Ord::cmp(b.0, a.0)) {
      if *block_height < reg_block_height {
        return Some(*point);
      }
    }
    None
  }
  pub(crate) fn detect_reorg(block: &BlockData, height: u32, index: &Index) -> Result {
    let bitcoind_prev_blockhash = block.header.prev_blockhash;

    match index.block_hash(height.checked_sub(1))? {
      Some(index_prev_blockhash) if index_prev_blockhash == bitcoind_prev_blockhash => Ok(()),
      Some(index_prev_blockhash) if index_prev_blockhash != bitcoind_prev_blockhash => {
        let max_recoverable_reorg_depth =
          (MAX_SAVEPOINTS - 1) * SAVEPOINT_INTERVAL + height % SAVEPOINT_INTERVAL;

        for depth in 1..max_recoverable_reorg_depth {
          let index_block_hash = index.block_hash(height.checked_sub(depth))?;
          let bitcoind_block_hash = index
            .client
            .get_block_hash(u64::from(height.saturating_sub(depth)))
            .into_option()?;

          if index_block_hash == bitcoind_block_hash {
            return Err(anyhow!(reorg::Error::Recoverable { height, depth }));
          }
        }

        Err(anyhow!(reorg::Error::Unrecoverable))
      }
      _ => Ok(()),
    }
  }

  pub(crate) fn handle_reorg(index: &Index, height: u32, depth: u32) -> Result {
    log::info!("rolling back database after reorg of depth {depth} at height {height}");

    if let redb::Durability::None = index.durability {
      panic!("set index durability to `Durability::Immediate` to test reorg handling");
    }

    let mut wtx = index.begin_write()?;
    let min_point_id = wtx.list_persistent_savepoints()?.min().unwrap();
    let reorg_height = (height - depth) as u64;
    let latest_point_id = Self::query_point(reorg_height).unwrap_or(min_point_id);
    log::info!("get latest point id: {latest_point_id} for height {reorg_height}");

    // remove the points which is largger than latest_point_id
    let should_remove_points = Self::retain_point(latest_point_id);
    for point in should_remove_points {
      log::info!("removing reorg points: {point}");
      wtx.delete_persistent_savepoint(point)?;
    }

    let savepoint = wtx.get_persistent_savepoint(latest_point_id)?;

    wtx.restore_savepoint(&savepoint)?;

    Index::increment_statistic(&wtx, Statistic::Commits, 1)?;
    wtx.commit()?;

    log::info!(
      "successfully rolled back database to height {}",
      index.begin_read()?.block_count()?
    );

    Ok(())
  }

  pub(crate) fn update_savepoints(index: &Index, height: u32) -> Result {
    if let redb::Durability::None = index.durability {
      return Ok(());
    }

    let height = u64::from(height);

    let last_savepoint_height = index
      .begin_read()?
      .0
      .open_table(STATISTIC_TO_COUNT)?
      .get(&Statistic::LastSavepointHeight.key())?
      .map(|last_savepoint_height| last_savepoint_height.value())
      .unwrap_or(0);

    // let blocks = index.client.get_blockchain_info()?.headers;
    let blocks = index.proxy_get_blockchain_info()?.headers;

    if (height < SAVEPOINT_INTERVAL.into()
      || height.saturating_sub(last_savepoint_height) >= SAVEPOINT_INTERVAL.into())
      && blocks.saturating_sub(height) <= CHAIN_TIP_DISTANCE.into()
    {
      let wtx = index.begin_write()?;

      let savepoints = wtx.list_persistent_savepoints()?.collect::<Vec<u64>>();

      if savepoints.len() >= usize::try_from(MAX_SAVEPOINTS).unwrap() {
        let point = savepoints.into_iter().min().unwrap();
        wtx.delete_persistent_savepoint(point)?;
        Self::remove_point(point);
      }
      wtx.commit()?;

      let wtx = index.begin_write()?;

      log::info!("creating savepoint at height {}", height);
      let point = wtx.persistent_savepoint()?;
      Self::insert_point(height, point);
      //wtx.persistent_savepoint()?;

      wtx
        .open_table(STATISTIC_TO_COUNT)?
        .insert(&Statistic::LastSavepointHeight.key(), &height)?;

      Index::increment_statistic(&wtx, Statistic::Commits, 1)?;
      wtx.commit()?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_reorg() {
    // (100,105] (105, 110]
    // 100 -> 0, 105 -> 1,
    for i in 0u64..=10 {
      Reorg::insert_point(100 + i * SAVEPOINT_INTERVAL as u64, i);
    }
    let reorg_height = 100;
    let point_index = Reorg::query_point(reorg_height);
    assert!(point_index.is_none());

    let reorg_height = 105;
    let point_index = Reorg::query_point(reorg_height).unwrap();
    assert_eq!(0, point_index);

    let reorg_height = 107;
    let point_index = Reorg::query_point(reorg_height).unwrap();
    assert_eq!(1, point_index);

    let reorg_height = 125;
    let point_index = Reorg::query_point(reorg_height).unwrap();
    assert_eq!(4, point_index);

    let reorg_height = 205;
    let point_index = Reorg::query_point(reorg_height);
    println!("{point_index:?}");
    assert_eq!(point_index, Some(10));

    // remove the points that larger than 5
    // keep 0,1,2,3,4,5
    show_points();
    let removed_points = Reorg::retain_point(5);
    println!("removed points: {:?}", removed_points);
    show_points();
  }

  pub fn show_points() {
    let points = POINTS.get_or_init(Default::default);
    println!("{:?}", points);
  }
}
