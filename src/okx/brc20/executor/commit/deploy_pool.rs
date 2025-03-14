use super::*;
use crate::okx::brc20::{
  entry::{BRC20ModulePoolPair, BRC20ModuleSwapPoolPairBalance},
  operation::commit::SwapFunctionData,
  verify::check_ticker_verify,
};

impl SwapFunctionData {
  pub(super) fn handle_deploy_pool(
    &self,
    module_id: &String,
    context: &mut TableContext,
  ) -> Result<(), ExecutionError> {
    log::info!("brc20swap handle_deploy_pool: {:?}", self);

    let params = self.params.clone().unwrap();
    let token0 = BRC20Ticker::from_str(&params[0]).unwrap();
    let token1 = BRC20Ticker::from_str(&params[1]).unwrap();

    let token0_amt = match check_ticker_verify(context, &token0, &String::new()) {
      Ok(token0_amt) => token0_amt,
      Err(_) => {
        log::debug!(
          "brc20swap error handle_deploy_pool: token0 {:?} not found",
          token0.clone().to_string()
        );
        return Err(ExecutionError::ExecutionFailed(BRC20Error::TickerNotFound(
          token0.clone().to_string(),
        )));
      }
    };

    let token1_amt = match check_ticker_verify(context, &token1, &String::new()) {
      Ok(token1_amt) => token1_amt,
      Err(_) => {
        log::debug!(
          "brc20swap error handle_deploy_pool: token1 {:?} not found",
          token1.clone().to_string()
        );
        return Err(ExecutionError::ExecutionFailed(BRC20Error::TickerNotFound(
          token1.clone().to_string(),
        )));
      }
    };

    let pool_pair = BRC20ModulePoolPair::new(&token0, &token1);

    if context
      .load_brc20_module_swap_poolpair_balance(&module_id, &pool_pair)?
      .is_some()
    {
      log::debug!(
        "brc20swap error handle_deploy_pool: pool_pair {}/{} already exists, inscription_id = {}",
        token0.clone().to_string(),
        token1.clone().to_string(),
        module_id.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::DuplicateDeployPool(pool_pair.clone().to_string()),
      ));
    }

    context.update_brc20_module_swap_poolpair_balance(
      &module_id,
      &pool_pair,
      BRC20ModuleSwapPoolPairBalance::new_with_balance(token0, token1, token0_amt, token1_amt),
    )?;

    log::debug!("brc20swap handle_deploy_pool finished: {:?}", self);
    Ok(())
  }
}
