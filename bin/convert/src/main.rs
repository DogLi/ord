use convert::check::check_inscription_table;
use convert::convert::{convert_entry_table, convert_inscription_table};
use redb::Database;

fn main() {
  let _ = env_logger::builder()
    .filter_level(log::LevelFilter::Info)
    .is_test(false)
    .try_init();
  // 获取命令行参数迭代器
  let args: Vec<String> = std::env::args().collect();
  if args.len() != 2 {
    log::warn!(
      "使用方法:\n./convert 1: 转换 inscription number 表\n./convert 2: 转换 convert 表\n3. 检查 inscription number 表"
    );
    return;
  }
  let batch_size = 50_000;
  let path = "/work/data/ord";
  let database = Database::builder()
    .create(&path)
    .expect("create database error");
  match args[1].as_str() {
    "1" => {
      if let Err(e) = convert_entry_table(&database, batch_size) {
        log::error!("转化 entry 失败: {e:?}");
        std::process::exit(1);
      }
    }
    "2" => {
      if let Err(e) = convert_inscription_table(&database, batch_size) {
        log::error!("装好 inscription 失败: {e:?}");
        std::process::exit(1);
      }
    }
    "check" => {
      log::info!("开始检查 inscription number 表");
      if let Err(e) = check_inscription_table(&database) {
        log::error!("检查 inscription number 表失败: {e:?}");
      }
    }
    _ => {
      log::warn!("使用方法:\n./convert 1: 转换 inscription number 表\n./convert 2: 转换 convert 表")
    }
  }
}
