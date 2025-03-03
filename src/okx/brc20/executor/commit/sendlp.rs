use std::cmp::Ordering;

use super::*;
use crate::okx::brc20::{
  brc20_decimal::Brc20Decimal,
  entry::{BRC20ModuleAddressLPTokenBalanceKey, BRC20ModulePoolPair},
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
    let token_amt_decimal = Brc20Decimal::from_str(&params[3]).map_err(|_e| {
      log::debug!("sendlp, decimal parse error: {}", params[3].clone());
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
            "sendlp, sender_lp_balance not exists: {}, {}",
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
        "sendlp: insufficient lp token balance: has: {:?}, wants: {:?}",
        sender_lp_balance.balance,
        token_amt_decimal
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InsufficientLPTokenBalance(sender_lp_balance.balance, token_amt_decimal),
      ));
    }

    let receiver_lp_key =
      BRC20ModuleAddressLPTokenBalanceKey::new(&module_id, &receiver, &pool_pair);
    let mut receiver_lp_balance =
      match context.load_brc20_module_address_lp_token_balance(&receiver_lp_key)? {
        Some(receiver_balance) => receiver_balance,
        None => {
          log::debug!(
            "sendlp, receiver_lp_balance not exists: {}, {}",
            module_id.clone(),
            pool_pair.to_string()
          );
          return Err(ExecutionError::ExecutionFailed(
            BRC20Error::ModuleBalanceNotExists(module_id.clone(), ticker_0.to_string()),
          ));
        }
      };

    sender_lp_balance.balance = sender_lp_balance.balance - token_amt_decimal.clone();
    receiver_lp_balance.balance = receiver_lp_balance.balance + token_amt_decimal.clone();

    context.update_brc20_module_address_lp_token_balance(&sender_lp_key, sender_lp_balance)?;
    context.update_brc20_module_address_lp_token_balance(&receiver_lp_key, receiver_lp_balance)?;

    Ok(())
  }
}
