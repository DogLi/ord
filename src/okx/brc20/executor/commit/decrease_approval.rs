use super::*;
use crate::okx::brc20::{brc20_decimal::Brc20Decimal, operation::commit::SwapFunctionData};
impl SwapFunctionData {
  pub(super) fn handle_decrease_approval(
    &self,
    chain: &Chain,
    module_id: &String,
    context: &mut TableContext,
  ) -> Result<(), ExecutionError> {
    log::info!("brc20swap handle_decrease_approval: {:?}", self);

    let params = self.params.clone().unwrap();
    let ticker = BRC20Ticker::from_str(&params[0]).unwrap();
    let ticker_amt_str = params[1].clone();
    let sender = UtxoAddress::from_str(self.address.as_ref().unwrap(), chain.network())
      .map_err(|e| ExecutionError::ExecutionFailed(BRC20Error::AddressParseError(e.to_string())))?;

    let ticker_info = match context.load_brc20_ticker_info(&ticker)? {
      Some(ticker_info) => {
        if ticker_info.ticker != ticker {
          log::debug!(
            "brc20swap error handle_decrease_approval: ticker_0 not found: {:?}",
            ticker.clone().to_string()
          );
          return Err(ExecutionError::ExecutionFailed(BRC20Error::TickerNotFound(
            ticker.clone().to_string(),
          )));
        }
        ticker_info
      }
      None => {
        log::debug!(
          "brc20swap error handle_decrease_approval none: ticker_0 not found: {:?}",
          ticker.clone().to_string()
        );
        return Err(ExecutionError::ExecutionFailed(BRC20Error::TickerNotFound(
          ticker.clone().to_string(),
        )));
      }
    };
    let ticker_amt = Brc20Decimal::from_str_with_scale(&ticker_amt_str, ticker_info.decimals)
      .map_err(|_e| {
        ExecutionError::ExecutionFailed(BRC20Error::DecimalParseError(ticker_amt_str.clone()))
      })?;
    if ticker_amt.sign() < 0 || ticker_amt.clone().to_u128().0 > ticker_info.total_supply {
      log::debug!(
        "brc20swap error handle_decrease_approval, token amount bigger than total supply: {}, total supply: {}",
        ticker_amt.clone(),
        ticker_info.total_supply.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InvalidTickerAmount(ticker_amt.clone().to_u128().0),
      ));
    }

    let mut module_balance =
      match context.load_brc20_module_address_token_balance(&sender, module_id, &ticker)? {
        Some(module_balance) => module_balance,
        None => {
          log::debug!(
            "brc20swap error handle_decrease_approval: module_balance {:?} {:?} not exists",
            module_id.clone(),
            ticker.clone(),
          );
          return Err(ExecutionError::ExecutionFailed(
            BRC20Error::ModuleBalanceNotExists(module_id.clone(), ticker.clone().to_string()),
          ));
        }
      };
    if module_balance.swap_account_balance < ticker_amt {
      log::debug!(
        "brc20swap error handle_decrease_approval: insufficient module balance: {:?}, {:?}",
        module_balance.swap_account_balance.clone(),
        ticker_amt.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InsufficientModuleBalance(module_balance.swap_account_balance, ticker_amt),
      ));
    }
    module_balance.swap_account_balance = module_balance.swap_account_balance - ticker_amt.clone();
    module_balance.available_balance = module_balance.available_balance + ticker_amt.clone();

    context.update_brc20_module_address_token_balance(
      &sender,
      module_id,
      &ticker,
      module_balance.clone(),
    )?;

    log::debug!("brc20swap decrease_approval finished: {:?}", self);
    Ok(())
  }
}
