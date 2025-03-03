use super::*;
use crate::okx::brc20::{
  brc20_decimal::Brc20Decimal, operation::commit::SwapFunctionData, verify::check_ticker_verify,
};
impl SwapFunctionData {
  pub(super) fn handle_decrease_approval(
    &self,
    sender: &UtxoAddress,
    module_id: &String,
    context: &mut TableContext,
  ) -> Result<(), ExecutionError> {
    log::info!("brc20swap handle_decrease_approval: {:?}", self);

    let params = self.params.clone().unwrap();
    let ticker = BRC20Ticker::from_str(&params[0]).unwrap();

    let token_amt = match check_ticker_verify(context, &ticker, &String::new()) {
      Ok(token_amt) => token_amt,
      Err(_) => {
        log::debug!(
          "brc20swap handle_decrease_approval: ticker {:?} not found",
          ticker.clone().to_string()
        );
        return Err(ExecutionError::ExecutionFailed(BRC20Error::TickerNotFound(
          ticker.clone().to_string(),
        )));
      }
    };
    let token_amt_decimal = Brc20Decimal::from_fix_point(token_amt);

    let mut module_balance =
      match context.load_brc20_module_address_token_balance(&sender, module_id, &ticker)? {
        Some(module_balance) => module_balance,
        None => {
          log::debug!(
            "brc20swap handle_decrease_approval: module_balance {:?} {:?} not exists",
            module_id.clone(),
            ticker.clone(),
          );
          return Err(ExecutionError::ExecutionFailed(
            BRC20Error::ModuleBalanceNotExists(module_id.clone(), ticker.clone().to_string()),
          ));
        }
      };
    if module_balance.swap_account_balance < token_amt_decimal {
      log::debug!(
        "brc20swap handle_decrease_approval: insufficient module balance: {:?}, {:?}",
        module_balance.swap_account_balance.clone(),
        token_amt_decimal.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InsufficientModuleBalance(
          module_balance.swap_account_balance,
          token_amt_decimal,
        ),
      ));
    }
    module_balance.swap_account_balance =
      module_balance.swap_account_balance - token_amt_decimal.clone();
    module_balance.available_balance = module_balance.available_balance + token_amt_decimal.clone();

    context.update_brc20_module_address_token_balance(
      &sender,
      module_id,
      &ticker,
      module_balance.clone(),
    )?;

    Ok(())
  }
}
