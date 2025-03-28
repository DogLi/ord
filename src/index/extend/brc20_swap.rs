use crate::okx::brc20::entry::{
  BRC20ModuleAddressLPTokenBalanceKey, BRC20ModuleAddressTokenBalanceKey, BRC20ModuleInfo,
  BRC20ModuleLPTokenBalance, BRC20ModulePoolPair, BRC20ModuleSwapPoolPairBalance,
  BRC20ModuleSwapPoolPairBalanceKey, BRC20ModuleTokenBalance, BRC20SwapInfo,
};

use super::*;

impl Index {
  pub(crate) fn get_module_info(module_id: &str, rtx: &Rtx) -> Result<Option<BRC20ModuleInfo>> {
    Ok(
      rtx
        .0
        .open_table(BRC20_MODULE_INFO)?
        .get(module_id)?
        .map(|v| BRC20ModuleInfo::load(v.value())),
    )
  }

  pub(crate) fn get_module_address_lp_balance(
    self: &Index,
    module_id: &str,
    address: &UtxoAddress,
    lp: &BRC20ModulePoolPair,
    rtx: &Rtx,
  ) -> Result<Option<BRC20ModuleLPTokenBalance>, Error> {
    let lp_key = BRC20ModuleAddressLPTokenBalanceKey::new(&module_id.to_string(), address, lp);

    Ok(
      rtx
        .0
        .open_table(BRC20_MODULE_ADDRESS_LP_TOKEN_BALANCE)?
        .get(lp_key.store().as_ref())?
        .map(|v| BRC20ModuleLPTokenBalance::load(v.value())),
    )
  }

  pub(crate) fn get_module_swap_pool_lp_balance(
    self: &Index,
    module_id: &str,
    pool_pair: &BRC20ModulePoolPair,
    rtx: &Rtx,
  ) -> Result<Option<BRC20ModuleSwapPoolPairBalance>, Error> {
    let lp_key = BRC20ModuleSwapPoolPairBalanceKey::new(&module_id.to_string(), pool_pair);

    Ok(
      rtx
        .0
        .open_table(BRC20_MODULE_SWAP_POOL_BALANCES)?
        .get(lp_key.store().as_ref())?
        .map(|v| BRC20ModuleSwapPoolPairBalance::load(v.value())),
    )
  }

  pub(crate) fn get_module_address_ticker_balance(
    module_id: &str,
    address: &UtxoAddress,
    ticker: &str,
    rtx: &Rtx,
  ) -> Result<Option<BRC20ModuleTokenBalance>> {
    Ok(
      rtx
        .0
        .open_table(BRC20_MODULE_ADDRESS_TICKER_BALANCE)?
        .get(
          BRC20ModuleAddressTokenBalanceKey {
            address: address.clone(),
            module_id: module_id.to_owned(),
            ticker: BRC20Ticker::from_str(ticker).unwrap().to_lowercase(),
          }
          .store()
          .as_ref(),
        )?
        .map(|v| BRC20ModuleTokenBalance::load(v.value())),
    )
  }
  pub(crate) fn get_brc20_swap_info(self: &Index, rtx: &Rtx) -> Result<BRC20SwapInfo, Error> {
    let total_module_count = rtx.0.open_table(BRC20_MODULE_INFO)?.len()?;
    let total_module_address_count = rtx
      .0
      .open_table(BRC20_MODULE_ADDRESS_TICKER_BALANCE)?
      .len()?;
    let total_module_swap_pool_address_count = rtx
      .0
      .open_table(BRC20_MODULE_ADDRESS_LP_TOKEN_BALANCE)?
      .len()?;
    let total_module_swap_pool_pair_count =
      rtx.0.open_table(BRC20_MODULE_SWAP_POOL_BALANCES)?.len()?;

    let all_modules = rtx
      .0
      .open_table(BRC20_MODULE_INFO)?
      .iter()?
      .map(|result| result.map(|(_, value)| BRC20ModuleInfo::load(value.value())))
      .collect::<Result<Vec<_>, _>>()?;

    Ok(BRC20SwapInfo {
      total_module_count,
      total_module_address_count,
      total_module_swap_pool_address_count,
      total_module_swap_pool_pair_count,
      all_modules,
    })
  }
}
