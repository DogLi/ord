use std::cmp::Ordering;

use super::*;
use crate::okx::brc20::{
  brc20_decimal::Brc20Decimal,
  entry::{BRC20ModuleAddressLPTokenBalanceKey, BRC20ModulePoolPair},
  operation::commit::SwapFunctionData,
};
impl SwapFunctionData {
  pub(super) fn handle_lock(
    &self,
    chain: &Chain,
    module_id: &String,
    context: &mut TableContext,
  ) -> Result<(), ExecutionError> {
    log::info!("brc20swap handle_lock: {:?}", self);

    let params = self.params.clone().unwrap();
    let ticker_0 = BRC20Ticker::from_str(&params[0])
      .map_err(|e| ExecutionError::ExecutionFailed(BRC20Error::TickerParse(e)))?;
    let ticker_1 = BRC20Ticker::from_str(&params[1])
      .map_err(|e| ExecutionError::ExecutionFailed(BRC20Error::TickerParse(e)))?;
    let pool_pair = BRC20ModulePoolPair::new(&ticker_0, &ticker_1);
    let sender: UtxoAddress =
      UtxoAddress::from_str(self.address.as_ref().unwrap(), chain.network()).map_err(|e| {
        ExecutionError::ExecutionFailed(BRC20Error::AddressParseError(e.to_string()))
      })?;
    let token_amt_decimal = Brc20Decimal::from_str_with_scale(&params[2], 18).map_err(|_e| {
      log::debug!(
        "brc20swap error handle_lock: token_amt_decimal parse error: {:?}",
        params[2].clone()
      );
      ExecutionError::ExecutionFailed(BRC20Error::DecimalParseError(params[2].clone()))
    })?;

    let lp_key = BRC20ModuleAddressLPTokenBalanceKey::new(&module_id, &sender, &pool_pair);

    match context.load_brc20_module_swap_poolpair_balance(&module_id, &pool_pair)? {
      Some(swap_pool_total_balance) => swap_pool_total_balance,
      None => {
        log::debug!(
          "brc20swap error handle_lock: pool_pair {:?} not exists",
          pool_pair.clone()
        );
        return Err(ExecutionError::ExecutionFailed(
          BRC20Error::SwapPoolTotalBalanceNotExists(pool_pair.to_string()),
        ));
      }
    };

    let mut lp_balance = match context.load_brc20_module_address_lp_token_balance(&lp_key)? {
      Some(lp_balance) => lp_balance,
      None => {
        log::debug!(
          "brc20swap error handle_lock: lp_key {:?} not exists",
          lp_key.clone()
        );
        return Err(ExecutionError::ExecutionFailed(
          BRC20Error::LPTokenBalanceNotExists(module_id.clone(), pool_pair.to_string()),
        ));
      }
    };

    if lp_balance.balance.cmp(&token_amt_decimal) == Ordering::Less {
      log::debug!(
        "brc20swap error handle_lock: insufficient lp token balance: has: {:?}, wants: {:?}",
        lp_balance.balance.clone(),
        token_amt_decimal.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InsufficientLPTokenBalance(lp_balance.balance, token_amt_decimal),
      ));
    }

    lp_balance.balance = lp_balance.balance - token_amt_decimal.clone();
    lp_balance.locked_balance = lp_balance.locked_balance + token_amt_decimal.clone();

    context.update_brc20_module_address_lp_token_balance(&lp_key, lp_balance)?;

    log::debug!("brc20swap handle_lock finished: {:?}", self);
    Ok(())
  }
}
