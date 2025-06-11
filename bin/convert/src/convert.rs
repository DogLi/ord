use crate::table::{
  INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER, INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER2,
  SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY, SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY2,
};
use redb::{Database, ReadableTable, TableDefinition};
use std::time::Instant;

/// 获取old table中多少条数据
fn get_data_len<K, V>(db: &Database, table_name: TableDefinition<K, V>) -> anyhow::Result<usize>
where
  K: redb::Key,
  V: redb::Value,
{
  let txn = db.begin_read()?;
  let table_old = txn.open_table(table_name)?;
  let batch_iter = table_old.iter();
  log::info!("开始获取表大小...");
  let now = Instant::now();
  let total_count = batch_iter.iter().count();
  log::info!(
    "获取表大小结束，共 {}w 数据， 用时: {:?}",
    total_count / 10000,
    now.elapsed()
  );
  if total_count == 0 {
    return Ok(1);
  }
  Ok(total_count)
}

pub fn convert_inscription_table(db: &Database, batch_size: usize) -> anyhow::Result<()> {
  let mut processed_count = 0;
  let total_count = get_data_len(db, INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER2)?;
  log::info!("Total inscription number count: {}", total_count);

  loop {
    let tx = db.begin_write()?;
    {
      let mut table_old = tx.open_table(INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER)?;
      let mut table_new = tx.open_table(INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER2)?;
      let mut batch_iter = table_old.iter().unwrap();
      let mut kv_list = Vec::with_capacity(batch_size);
      for _ in 0..batch_size {
        if let Some(item) = batch_iter.next() {
          let (k, v) = item.unwrap();
          kv_list.push((k.value(), v.value()));
        }
      }
      if kv_list.is_empty() {
        log::info!("inscription number 数据处理完毕");
        break;
      }
      for (k, v) in kv_list {
        table_new.insert(k as i64, v)?;
        table_old.remove(&k)?;
        processed_count += 1;
      }
    }
    tx.commit()?;
    log::info!(
      "迁移数据 {}w, total:{}w --> {}%",
      processed_count / 10000,
      total_count / 10000,
      (processed_count as f64 / total_count as f64) * 100.0
    );
  }
  Ok(())
}

pub fn convert_entry_table(db: &Database, batch_size: usize) -> anyhow::Result<()> {
  let mut processed_count = 0;
  let total_count = get_data_len(db, SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;
  log::info!("Total entry count: {}", total_count);

  loop {
    let tx = db.begin_write()?;
    {
      let mut table_old = tx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;
      let mut table_new = tx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY2)?;
      let mut batch_iter = table_old.iter().unwrap();
      let mut kv_list = Vec::with_capacity(batch_size);
      for _ in 0..batch_size {
        if let Some(item) = batch_iter.next() {
          let (k, v) = item.unwrap();
          kv_list.push((k.value(), v.value()));
        }
      }
      if kv_list.is_empty() {
        log::info!("entry 数据处理完毕");
        break;
      }
      for (k, v) in kv_list {
        let v2 = (v.0, v.1, v.2, v.3, v.4 as i64, v.5, v.6, v.7, v.8);
        table_new.insert(k, v2)?;
        table_old.remove(&k)?;
        processed_count += 1;
      }
    }
    tx.commit()?;
    log::info!(
      "entry 迁移数据 {}w, total:{}w --> {}%",
      processed_count / 10000,
      total_count / 10000,
      (processed_count as f64 / total_count as f64) * 100.0
    );
  }
  Ok(())
}
