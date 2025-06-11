use crate::table::INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER2;
use redb::Database;

/// 检查 inscription number 表数据是否一致
pub fn check_inscription_table(db: &Database) -> anyhow::Result<()> {
  let tx = db.begin_read()?;
  let table = tx.open_table(INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER2)?;
  for i in 2_000_000_000_u32..2_100_000_000 {
    let v = table.get(i as i64)?;
    if v.is_none() {
      break;
    }
    let v = v.unwrap();
    let v = v.value();
    if v != i {
      log::error!("inscription number 表数据不一致: {} != {}", i, v);
      break;
    }
  }
  log::info!("检查完毕，数据一致");
  Ok(())
}
