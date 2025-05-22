use std::{
  cmp::Ordering,
  ops::{Div, Mul, Sub},
};

use super::*;
use crate::okx::brc20::{
  brc20_decimal::Brc20Decimal,
  entry::{
    BRC20ModuleAddressLPTokenBalanceKey, BRC20ModuleLPTokenBalance, BRC20ModulePoolPair,
    BRC20ModuleTokenBalance,
  },
  operation::commit::SwapFunctionData,
};
impl SwapFunctionData {
  pub(super) fn handle_remove_liq(
    &self,
    chain: &Chain,
    module_id: &String,
    context: &mut TableContext,
    fee_rate: Brc20Decimal,
  ) -> Result<(), ExecutionError> {
    log::info!("brc20swap handle_remove_liq: {:?}", self);

    // load params
    let sender = UtxoAddress::from_str(self.address.as_ref().unwrap(), chain.network())
      .map_err(|e| ExecutionError::ExecutionFailed(BRC20Error::AddressParseError(e.to_string())))?;
    let params = self.params.clone().unwrap();
    if params.len() != 6 {
      log::debug!(
        "brc20swap error handle_remove_liq: invalid params: {:?}, length: {:?}",
        params.clone(),
        params.len()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InvalidParamsLength(params.len(), 6),
      ));
    }
    let ticker_0 = BRC20Ticker::from_str(&params[0])
      .map_err(|e| ExecutionError::ExecutionFailed(BRC20Error::TickerParse(e)))?;
    let ticker_1 = BRC20Ticker::from_str(&params[1])
      .map_err(|e| ExecutionError::ExecutionFailed(BRC20Error::TickerParse(e)))?;
    let pool_pair = BRC20ModulePoolPair::new(&ticker_0, &ticker_1);
    let token_lp_amt = Brc20Decimal::from_str_with_scale(&params[2], 18).map_err(|_e| {
      ExecutionError::ExecutionFailed(BRC20Error::DecimalParseError(params[2].clone()))
    })?;
    let token_0 = &params[3];
    let token_1 = &params[4];
    let slippage_amt = Brc20Decimal::from_str_with_scale(&params[5], 3).map_err(|_e| {
      ExecutionError::ExecutionFailed(BRC20Error::DecimalParseError(params[5].clone()))
    })?;

    // verify params
    let ticker_0_info = match context.load_brc20_ticker_info(&ticker_0)? {
      Some(ticker_info) => {
        if ticker_info.ticker != ticker_0 {
          log::debug!(
            "brc20swap error remove_liq, ticker_0 not found: {}",
            ticker_0.clone().to_string()
          );
          return Err(ExecutionError::ExecutionFailed(BRC20Error::TickerNotFound(
            ticker_0.clone().to_string(),
          )));
        }
        ticker_info
      }
      None => {
        log::debug!(
          "brc20swap error remove_liq none, ticker_0 not found: {}",
          ticker_0.clone().to_string()
        );
        return Err(ExecutionError::ExecutionFailed(BRC20Error::TickerNotFound(
          ticker_0.clone().to_string(),
        )));
      }
    };
    let token_0_amt =
      Brc20Decimal::from_str_with_scale(token_0, ticker_0_info.decimals).map_err(|_e| {
        ExecutionError::ExecutionFailed(BRC20Error::DecimalParseError(token_0.clone()))
      })?;
    if token_0_amt.sign() < 0 || token_0_amt.clone().to_u128().0 > ticker_0_info.total_supply {
      log::debug!(
        "brc20swap error remove_liq, token amount bigger than total supply: {}, total supply: {}",
        token_0_amt.clone(),
        ticker_0_info.total_supply.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InvalidTickerAmount(token_0_amt.clone().to_u128().0),
      ));
    }
    let ticker_1_info = match context.load_brc20_ticker_info(&ticker_1)? {
      Some(ticker_info) => {
        if ticker_info.ticker != ticker_1 {
          log::debug!(
            "brc20swap error remove_liq, ticker_1 not found: {}",
            ticker_1.clone().to_string()
          );
          return Err(ExecutionError::ExecutionFailed(BRC20Error::TickerNotFound(
            ticker_1.clone().to_string(),
          )));
        }
        ticker_info
      }
      None => {
        log::debug!(
          "brc20swap error remove_liq none, ticker_1 not found: {}",
          ticker_1.clone().to_string()
        );
        return Err(ExecutionError::ExecutionFailed(BRC20Error::TickerNotFound(
          ticker_1.clone().to_string(),
        )));
      }
    };
    let token_1_amt =
      Brc20Decimal::from_str_with_scale(token_1, ticker_1_info.decimals).map_err(|_e| {
        ExecutionError::ExecutionFailed(BRC20Error::DecimalParseError(token_1.clone()))
      })?;
    if token_1_amt.sign() < 0 || token_1_amt.clone().to_u128().0 > ticker_1_info.total_supply {
      log::debug!(
        "brc20swap error remove_liq, token amount bigger than total supply: {}, total supply: {}",
        token_1_amt.clone(),
        ticker_1_info.total_supply.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InvalidTickerAmount(token_1_amt.clone().to_u128().0),
      ));
    }

    let mut pool_balance =
      match context.load_brc20_module_swap_poolpair_balance(&module_id, &pool_pair)? {
        Some(pool) => pool,
        None => {
          log::debug!(
            "brc20swap error remove_liq, pool_pair not found: {}",
            pool_pair.clone().to_string()
          );
          return Err(ExecutionError::ExecutionFailed(
            BRC20Error::SwapPoolTotalBalanceNotExists(pool_pair.to_string()),
          ));
        }
      };

    let token_0_idx: usize;
    let token_1_idx: usize;
    if pool_balance.tick[0] == ticker_0 {
      token_0_idx = 0;
      token_1_idx = 1;
    } else {
      token_0_idx = 1;
      token_1_idx = 0;
    }

    let module_info = match context.load_brc20_module_info(&module_id)? {
      Some(module_info) => module_info,
      None => {
        log::debug!(
          "brc20swap error remove_liq none, module_id not found: {}",
          module_id.clone()
        );
        return Err(ExecutionError::ExecutionFailed(
          BRC20Error::ModuleNotExists(module_id.clone()),
        ));
      }
    };

    // Increase LP, as a method of collecting service fees.
    if fee_rate.sign() > 0 {
      // lp = (poolLp * (rootK - rootKLast)) / (rootK * 5 + rootKLast)
      let root_k = pool_balance.tick_balance[token_0_idx]
        .clone()
        .mul(pool_balance.tick_balance[token_1_idx].clone())
        .sqrt()
        .unwrap();
      let lp_fee = pool_balance
        .lp_balance
        .clone()
        .mul(root_k.clone() - pool_balance.last_root_k.clone())
        / (root_k.clone().mul(Brc20Decimal::from_u128(5, 0).unwrap())
          + pool_balance.last_root_k.clone());
      if lp_fee.sign() > 0 {
        // pool lp update
        pool_balance.lp_balance = pool_balance.lp_balance + lp_fee.clone();

        // lp_fee update
        let lp_fee_address = UtxoAddress::from_script(
          ScriptBuf::from_hex(&module_info.lp_fee_pk_script.clone())
            .unwrap()
            .as_script(),
          chain,
        );
        let lp_fee_key =
          BRC20ModuleAddressLPTokenBalanceKey::new(module_id, &lp_fee_address, &pool_pair);
        let mut lp_fee_token_balance = context
          .load_brc20_module_address_lp_token_balance(&lp_fee_key)?
          .unwrap_or(BRC20ModuleLPTokenBalance::new_with_default());
        lp_fee_token_balance.balance = lp_fee_token_balance.balance + lp_fee.clone();
        context.update_brc20_module_address_lp_token_balance(&lp_fee_key, lp_fee_token_balance)?;
      }
    }

    // slippage check
    let amt_0 = pool_balance.tick_balance[token_0_idx]
      .clone()
      .mul(token_lp_amt.clone())
      .div(pool_balance.lp_balance.clone());
    if amt_0.cmp(
      &token_0_amt
        .clone()
        .sub(token_0_amt.clone().mul(slippage_amt.clone())),
    ) == Ordering::Less
    {
      log::debug!(
        "brc20swap error remove_liq over slippage, sender: {:?}, token0: {:?}, expect: {:?}",
        sender.clone(),
        amt_0.clone(),
        token_0_amt.clone(),
      );
      return Err(ExecutionError::ExecutionFailed(BRC20Error::OverSlippage()));
    }
    let amt_1 = pool_balance.tick_balance[token_1_idx]
      .clone()
      .mul(token_lp_amt.clone())
      .div(pool_balance.lp_balance.clone());
    if amt_1.cmp(
      &token_1_amt
        .clone()
        .sub(token_1_amt.clone().mul(slippage_amt.clone())),
    ) == Ordering::Less
    {
      log::debug!(
        "brc20swap error remove_liq over slippage, sender: {:?}, token1: {:?}, expect: {:?}",
        sender.clone(),
        amt_1.clone(),
        token_1_amt.clone(),
      );
      return Err(ExecutionError::ExecutionFailed(BRC20Error::OverSlippage()));
    }

    // change in pool balance
    if pool_balance.lp_balance.cmp(&token_lp_amt.clone()) == Ordering::Less {
      log::debug!(
        "brc20swap error remove_liq insufficient lp token balance, sender: {:?}, pool_lp: {:?}, token_lp: {:?}",
        sender.clone(),
        pool_balance.lp_balance.clone(),
        token_lp_amt.clone(),
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InsufficientLPTokenBalance(
          pool_balance.lp_balance.clone(),
          token_lp_amt.clone(),
        ),
      ));
    }
    if pool_balance.tick_balance[token_0_idx].cmp(&amt_0.clone()) == Ordering::Less {
      log::debug!(
        "brc20swap error remove_liq insufficient pool balance, sender: {:?}, pool: {:?}",
        sender.clone(),
        pool_pair.clone(),
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InsufficientPoolBalance(
          pool_balance.tick_balance[token_0_idx].clone(),
          amt_0.clone(),
        ),
      ));
    }
    if pool_balance.tick_balance[token_1_idx].cmp(&amt_1.clone()) == Ordering::Less {
      log::debug!(
        "brc20swap error remove_liq insufficient pool balance, sender: {:?}, pool: {:?}",
        sender.clone(),
        pool_pair.clone(),
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InsufficientPoolBalance(
          pool_balance.tick_balance[token_1_idx].clone(),
          amt_1.clone(),
        ),
      ));
    }

    let lp_key = BRC20ModuleAddressLPTokenBalanceKey::new(module_id, &sender, &pool_pair);
    let mut user_lp_balance = context
      .load_brc20_module_address_lp_token_balance(&lp_key)?
      .unwrap_or(BRC20ModuleLPTokenBalance::new_with_default());
    if user_lp_balance.balance.cmp(&token_lp_amt.clone()) == Ordering::Less {
      log::debug!(
        "brc20swap error remove_liq insufficient lp token balance, sender: {:?}, pool_lp: {:?}, token_lp: {:?}",
        sender.clone(),
        user_lp_balance.balance.clone(),
        token_lp_amt.clone(),
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InsufficientLPTokenBalance(
          user_lp_balance.balance.clone(),
          token_lp_amt.clone(),
        ),
      ));
    }

    // remove liq
    user_lp_balance.balance = user_lp_balance.balance - token_lp_amt.clone();
    context.update_brc20_module_address_lp_token_balance(&lp_key, user_lp_balance)?;

    let mut token_0_balance = context
      .load_brc20_module_address_token_balance(&sender, module_id, &ticker_0)?
      .unwrap_or(BRC20ModuleTokenBalance::new_with_scale(
        ticker_0_info.decimals,
      ));
    let mut token_1_balance = context
      .load_brc20_module_address_token_balance(&sender, module_id, &ticker_1)?
      .unwrap_or(BRC20ModuleTokenBalance::new_with_scale(
        ticker_1_info.decimals,
      ));

    token_0_balance.swap_account_balance = token_0_balance.swap_account_balance + amt_0.clone();
    token_1_balance.swap_account_balance = token_1_balance.swap_account_balance + amt_1.clone();

    pool_balance.lp_balance = pool_balance.lp_balance - token_lp_amt.clone();

    // deduct token balance in the pool
    pool_balance.tick_balance[token_0_idx] =
      pool_balance.tick_balance[token_0_idx].clone() - amt_0.clone();
    pool_balance.tick_balance[token_1_idx] =
      pool_balance.tick_balance[token_1_idx].clone() - amt_1.clone();

    pool_balance.last_root_k = pool_balance.tick_balance[token_0_idx]
      .clone()
      .mul(pool_balance.tick_balance[token_1_idx].clone())
      .sqrt()
      .unwrap();

    context.update_brc20_module_address_token_balance(
      &sender,
      module_id,
      &ticker_0,
      token_0_balance,
    )?;
    context.update_brc20_module_address_token_balance(
      &sender,
      module_id,
      &ticker_1,
      token_1_balance,
    )?;
    context.update_brc20_module_swap_poolpair_balance(&module_id, &pool_pair, pool_balance)?;

    log::debug!("brc20swap handle_remove_liq finished: {:?}", self);
    Ok(())
  }
}
