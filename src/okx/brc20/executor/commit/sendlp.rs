use std::cmp::Ordering;

use super::*;
use crate::okx::brc20::{
  brc20_decimal::Brc20Decimal,
  entry::{BRC20ModuleAddressLPTokenBalanceKey, BRC20ModuleLPTokenBalance, BRC20ModulePoolPair},
  operation::commit::SwapFunctionData,
};
impl SwapFunctionData {
  pub(super) fn handle_send_lp(
    &self,
    chain: &Chain,
    module_id: &String,
    context: &mut TableContext,
  ) -> Result<(), ExecutionError> {
    log::info!("brc20swap handle_send_lp: {:?}", self);

    let params = self.params.clone().unwrap();
    let receiver = UtxoAddress::from_str(&params[0], chain.network())
      .map_err(|e| ExecutionError::ExecutionFailed(BRC20Error::AddressParseError(e.to_string())))?;
    let ticker_0 = BRC20Ticker::from_str(&params[1])
      .map_err(|e| ExecutionError::ExecutionFailed(BRC20Error::TickerParse(e)))?;
    let ticker_1 = BRC20Ticker::from_str(&params[2])
      .map_err(|e| ExecutionError::ExecutionFailed(BRC20Error::TickerParse(e)))?;
    let pool_pair = BRC20ModulePoolPair::new(&ticker_0, &ticker_1);
    let token_amt_decimal = Brc20Decimal::from_str_with_scale(&params[3], 18).map_err(|_e| {
      log::debug!(
        "brc20swap error handle_send_lp: decimal parse error: {}, {}",
        params[3].clone(),
        self.address.as_ref().unwrap()
      );
      ExecutionError::ExecutionFailed(BRC20Error::DecimalParseError(params[3].clone()))
    })?;
    let sender: UtxoAddress =
      UtxoAddress::from_str(self.address.as_ref().unwrap(), chain.network()).map_err(|e| {
        ExecutionError::ExecutionFailed(BRC20Error::AddressParseError(e.to_string()))
      })?;

    let sender_lp_key = BRC20ModuleAddressLPTokenBalanceKey::new(&module_id, &sender, &pool_pair);
    let mut sender_lp_balance =
      match context.load_brc20_module_address_lp_token_balance(&sender_lp_key)? {
        Some(lp_token_balance) => lp_token_balance,
        None => {
          log::debug!(
            "brc20swap error handle_send_lp: sender_lp_balance not exists: {}, {}",
            module_id.clone(),
            pool_pair.to_string()
          );
          return Err(ExecutionError::ExecutionFailed(
            BRC20Error::LPTokenBalanceNotExists(module_id.clone(), pool_pair.to_string()),
          ));
        }
      };
    if sender_lp_balance.balance.cmp(&token_amt_decimal) == Ordering::Less {
      log::debug!(
        "brc20swap error handle_send_lp: insufficient lp token balance: has: {:?}, wants: {:?}",
        sender_lp_balance.balance,
        token_amt_decimal
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InsufficientLPTokenBalance(sender_lp_balance.balance, token_amt_decimal),
      ));
    }
    sender_lp_balance.balance = sender_lp_balance.balance - token_amt_decimal.clone();
    context.update_brc20_module_address_lp_token_balance(&sender_lp_key, sender_lp_balance)?;

    let receiver_lp_key =
      BRC20ModuleAddressLPTokenBalanceKey::new(&module_id, &receiver, &pool_pair);
    let mut receiver_lp_balance =
      match context.load_brc20_module_address_lp_token_balance(&receiver_lp_key)? {
        Some(receiver_balance) => receiver_balance,
        None => BRC20ModuleLPTokenBalance::new(
          Brc20Decimal::from_u128(0, token_amt_decimal.get_precision()).unwrap(),
          Brc20Decimal::from_u128(0, token_amt_decimal.get_precision()).unwrap(),
        ),
      };

    receiver_lp_balance.balance = receiver_lp_balance.balance + token_amt_decimal.clone();
    context.update_brc20_module_address_lp_token_balance(&receiver_lp_key, receiver_lp_balance)?;

    log::debug!("brc20swap handle_send_lp finished: {:?}", self);
    Ok(())
  }
}
