use crate::okx::{
  brc20::{
    entry::{BRC20ModuleInfo, BRC20ModulePoolPair, BRC20ModuleTokenBalance},
    BRC20Ticker,
  },
  UtxoAddress,
};

use super::*;

pub(crate) async fn get_module_info(
  Extension(index): Extension<Arc<Index>>,
  Path(module_id): Path<String>,
) -> Result<Json<BRC20ModuleInfo>, ServerError> {
  log::debug!("rpc: get_module_info: {}", module_id);

  let rtx = index.begin_read()?;
  let module_info = match Index::get_module_info(module_id.as_str(), &rtx) {
    Ok(Some(module_info)) => module_info,
    Ok(_) => {
      return Err(ServerError::NotFound(format!(
        "module not found: {}",
        module_id
      )))
    }
    Err(e) => return Err(ServerError::Internal(e.into())),
  };

  Ok(Json(module_info))
}

pub(crate) async fn get_module_ticker_balance(
  Extension(settings): Extension<Arc<Settings>>,
  Extension(index): Extension<Arc<Index>>,
  Path((module_id, address, ticker)): Path<(String, String, String)>,
) -> Result<Json<BRC20ModuleTokenBalance>, ServerError> {
  log::debug!(
    "rpc: get_module_ticker_balance: module_id:{}, address:{}, ticker:{}",
    module_id,
    address,
    ticker
  );

  let rtx = index.begin_read()?;

  let utxo_address = UtxoAddress::from_str(&address, settings.chain().network())
    .map_err(|_| ServerError::NotFound(format!("invalid address: {}", address)))?;

  let brc20_module_token_balance = match Index::get_module_address_ticker_balance(
    module_id.as_str(),
    &utxo_address,
    &ticker,
    &rtx,
  ) {
    Ok(Some(module_info)) => module_info,
    Ok(_) => return Err(ServerError::NotFound(module_id.into())),
    Err(e) => return Err(ServerError::Internal(e.into())),
  };

  Ok(Json(brc20_module_token_balance))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LpBalance {
  pub module_id: String,
  pub lp: String,
  pub balance: String,
  pub locked_balance: String,
}

pub(crate) async fn get_module_address_lp_balance(
  Extension(settings): Extension<Arc<Settings>>,
  Extension(index): Extension<Arc<Index>>,
  Path((module_id, address, ticker_1, ticker_2)): Path<(String, String, String, String)>,
) -> Result<Json<LpBalance>, ServerError> {
  log::debug!(
    "rest: get_module_lp_balance, module_id: {}, ticker_1: {}, ticker_2: {}",
    module_id,
    ticker_1,
    ticker_2
  );

  let address = UtxoAddress::from_str(&address, settings.chain().network())
    .map_err(|_e| ServerError::BadRequest(format!("invalid address: {}", address)))?;

  let ticker_1 = BRC20Ticker::from_str(&ticker_1)
    .map_err(|_e| ServerError::BadRequest(format!("invalid ticker_1: {}", ticker_1)))?;
  let ticker_2 = BRC20Ticker::from_str(&ticker_2)
    .map_err(|_e| ServerError::BadRequest(format!("invalid ticker_2: {}", ticker_2)))?;

  let lp = BRC20ModulePoolPair::new(&ticker_1, &ticker_2);

  let rtx = index.begin_read()?;
  let module_lp_balance =
    match Index::get_module_address_lp_balance(&index, module_id.as_str(), &address, &lp, &rtx) {
      Ok(Some(module_lp_balance)) => module_lp_balance,
      Ok(_) => {
        return Err(ServerError::NotFound(String::from(
          "module, address and lp combination result not found",
        )))
      }
      Err(e) => return Err(ServerError::Internal(e.into())),
    };

  Ok(Json(LpBalance {
    module_id,
    lp: lp.to_string(),
    balance: module_lp_balance.balance.to_string(),
    locked_balance: module_lp_balance.locked_balance.to_string(),
  }))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapPoolLpBalance {
  pub module_id: String,
  pub lp: String,
  pub ticker_0: String,
  pub ticker_1: String,
  pub ticker_0_balance: String,
  pub ticker_1_balance: String,
  pub lp_balance: String,
  pub last_root_k: String,
}

pub(crate) async fn get_module_swap_pool_lp_balance(
  Extension(index): Extension<Arc<Index>>,
  Path((module_id, ticker_0, ticker_1)): Path<(String, String, String)>,
) -> Result<Json<SwapPoolLpBalance>, ServerError> {
  log::debug!(
    "rest: get_module_lp_balance, module_id: {}, ticker_1: {}, ticker_2: {}",
    module_id,
    ticker_0,
    ticker_1
  );

  let ticker_0 = BRC20Ticker::from_str(&ticker_0)
    .map_err(|_e| ServerError::BadRequest(format!("invalid ticker_1: {}", ticker_0)))?;
  let ticker_1 = BRC20Ticker::from_str(&ticker_1)
    .map_err(|_e| ServerError::BadRequest(format!("invalid ticker_2: {}", ticker_0)))?;
  let lp = BRC20ModulePoolPair::new(&ticker_0, &ticker_1);

  let rtx = index.begin_read()?;
  let module_swap_pool_lp_balance =
    match Index::get_module_swap_pool_lp_balance(&index, module_id.as_str(), &lp, &rtx) {
      Ok(Some(module_swap_pool_lp_balance)) => module_swap_pool_lp_balance,
      Ok(_) => {
        return Err(ServerError::NotFound(String::from(
          "module, address and lp combination result not found",
        )))
      }
      Err(e) => return Err(ServerError::Internal(e.into())),
    };

  log::debug!(
    "rest: get_module_swap_pool_lp_balance, module_id: {}, ticker_0: {}, ticker_1: {}, lp: {:?}, lp_string: {:?}",
    module_id,
    ticker_0,
    ticker_1,
    lp,
    lp.to_string(),
  );

  Ok(Json(SwapPoolLpBalance {
    module_id,
    lp: lp.to_string(),
    ticker_0: module_swap_pool_lp_balance.tick[0].to_string(),
    ticker_1: module_swap_pool_lp_balance.tick[1].to_string(),
    ticker_0_balance: module_swap_pool_lp_balance.tick_balance[0].to_string(),
    ticker_1_balance: module_swap_pool_lp_balance.tick_balance[1].to_string(),
    lp_balance: module_swap_pool_lp_balance.lp_balance.to_string(),
    last_root_k: module_swap_pool_lp_balance.last_root_k.to_string(),
  }))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Brc20SwapInfo {
  pub total_module_count: u64,
  pub total_module_address_count: u64,
  pub total_module_swap_pool_address_count: u64,
  pub total_module_swap_pool_pair_count: u64,
  pub all_modules: Vec<BRC20ModuleInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Brc20SwapInfoParams {
  pub without_op_return_address: bool,
}

pub(crate) async fn get_brc20_swap_info(
  Extension(index): Extension<Arc<Index>>,
  Query(params): Query<Brc20SwapInfoParams>,
) -> Result<Json<Brc20SwapInfo>, ServerError> {
  let rtx = index.begin_read()?;

  let swap_info = match Index::get_brc20_swap_info(&index, &rtx, params.without_op_return_address) {
    Ok(swap_info) => swap_info,
    Err(e) => return Err(ServerError::Internal(e.into())),
  };

  Ok(Json(Brc20SwapInfo {
    total_module_count: swap_info.total_module_count,
    total_module_address_count: swap_info.total_module_address_count,
    total_module_swap_pool_address_count: swap_info.total_module_swap_pool_address_count,
    total_module_swap_pool_pair_count: swap_info.total_module_swap_pool_pair_count,
    all_modules: swap_info.all_modules,
  }))
}
