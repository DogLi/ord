use super::*;
use crate::okx::brc20::{brc20_decimal::Brc20Decimal, operation::commit::SwapFunctionData};
impl SwapFunctionData {
  pub(super) fn handle_send(
    &self,
    chain: &Chain,
    module_id: &String,
    context: &mut TableContext,
  ) -> Result<(), ExecutionError> {
    log::info!("brc20swap handle_send: {:?}", self);

    let params = self.params.clone().unwrap();
    let receiver = UtxoAddress::from_str(&params[0], chain.network())
      .map_err(|e| ExecutionError::ExecutionFailed(BRC20Error::AddressParseError(e.to_string())))?;
    let ticker = BRC20Ticker::from_str(&params[1])
      .map_err(|e| ExecutionError::ExecutionFailed(BRC20Error::TickerParse(e)))?;
    let amount_str = &params[2];
    let sender: UtxoAddress =
      UtxoAddress::from_str(self.address.as_ref().unwrap(), chain.network()).map_err(|e| {
        log::debug!(
          "brc20swap error handle_send: address parse error: {}, {}",
          e.to_string(),
          self.address.as_ref().unwrap()
        );
        ExecutionError::ExecutionFailed(BRC20Error::AddressParseError(e.to_string()))
      })?;

    let ticker_info = match context.load_brc20_ticker_info(&ticker)? {
      Some(ticker_info) => ticker_info,
      None => {
        log::debug!(
          "brc20swap error handle_send, ticker not found: {}",
          ticker.clone().to_string()
        );
        return Err(ExecutionError::ExecutionFailed(BRC20Error::TickerNotFound(
          ticker.clone().to_string(),
        )));
      }
    };
    let amount =
      Brc20Decimal::from_str_with_scale(amount_str, ticker_info.decimals).map_err(|_e| {
        ExecutionError::ExecutionFailed(BRC20Error::DecimalParseError(amount_str.clone()))
      })?;
    if amount.sign() < 0 || amount.clone().to_u128().0 > ticker_info.total_supply {
      log::debug!(
        "brc20swap error handle_send, token amount bigger than total supply: {}, total supply: {}",
        amount,
        ticker_info.total_supply
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InvalidTickerAmount(amount.clone().to_u128().0),
      ));
    }

    let mut sender_balance =
      match context.load_brc20_module_address_token_balance(&sender, module_id, &ticker)? {
        Some(sender_balance) => sender_balance,
        None => {
          log::debug!(
            "brc20swap error handle_send, module balance not exists: {}, {}",
            module_id.clone(),
            ticker.clone().to_string()
          );
          return Err(ExecutionError::ExecutionFailed(
            BRC20Error::ModuleBalanceNotExists(module_id.clone(), ticker.to_string()),
          ));
        }
      };
    if sender_balance.swap_account_balance < amount {
      log::debug!(
        "brc20swap error handle_send: insufficient module balance: has: {:?}, wants: {:?}",
        sender_balance.swap_account_balance.clone(),
        amount.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InsufficientModuleBalance(sender_balance.swap_account_balance, amount),
      ));
    }
    sender_balance.swap_account_balance = sender_balance.swap_account_balance - amount.clone();
    context.update_brc20_module_address_token_balance(
      &sender,
      module_id,
      &ticker,
      sender_balance,
    )?;

    let mut receiver_balance =
      match context.load_brc20_module_address_token_balance(&receiver, module_id, &ticker)? {
        Some(receiver_balance) => receiver_balance,
        None => BRC20ModuleTokenBalance::new_with_scale(amount.clone().get_precision()),
      };

    receiver_balance.swap_account_balance = receiver_balance.swap_account_balance + amount.clone();
    context.update_brc20_module_address_token_balance(
      &receiver,
      module_id,
      &ticker,
      receiver_balance,
    )?;

    log::debug!("brc20swap handle_send finished: {:?}", self);
    Ok(())
  }
}
