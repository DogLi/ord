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
  pub(super) fn handle_add_liq(
    &self,
    chain: &Chain,
    module_id: &String,
    context: &mut TableContext,
    fee_rate: Brc20Decimal,
  ) -> Result<(), ExecutionError> {
    log::info!("brc20swap handle_add_liq: {:?}", self);

    // load params
    let sender = UtxoAddress::from_str(self.address.as_ref().unwrap(), chain.network())
      .map_err(|e| ExecutionError::ExecutionFailed(BRC20Error::AddressParseError(e.to_string())))?;
    let params = self.params.clone().unwrap();
    if params.len() != 6 {
      log::debug!(
        "brc20swap error handle_add_liq: params length is not 6: {:?}, params: {:?}",
        params.len(),
        params.clone()
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
    let token_0 = &params[2];
    let token_1 = &params[3];
    let expected_lp_amt = Brc20Decimal::from_str_with_scale(&params[4], 18).map_err(|_e| {
      ExecutionError::ExecutionFailed(BRC20Error::DecimalParseError(params[4].clone()))
    })?;
    let slippage_amt = Brc20Decimal::from_str_with_scale(&params[5], 3).map_err(|_e| {
      ExecutionError::ExecutionFailed(BRC20Error::DecimalParseError(params[5].clone()))
    })?;

    // verify params
    let ticker_0_info = match context.load_brc20_ticker_info(&ticker_0)? {
      Some(ticker_info) => {
        if ticker_info.ticker != ticker_0 {
          log::debug!(
            "brc20swap error handle_add_liq: ticker_0 not found: {:?}",
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
          "brc20swap error handle_add_liq none: ticker_0 not found: {:?}",
          ticker_0.clone().to_string()
        );
        return Err(ExecutionError::ExecutionFailed(BRC20Error::TickerNotFound(
          ticker_0.clone().to_string(),
        )));
      }
    };
    let mut token_0_amt = Brc20Decimal::from_str_with_scale(token_0, ticker_0_info.decimals)
      .map_err(|_e| {
        ExecutionError::ExecutionFailed(BRC20Error::DecimalParseError(token_0.clone()))
      })?;
    if token_0_amt.sign() < 0 || token_0_amt.clone().to_u128().0 > ticker_0_info.total_supply {
      log::debug!(
        "brc20swap error add_liq, token0 amount bigger than total supply: {}, total supply: {}",
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
            "brc20swap error handle_add_liq: ticker_1 not found: {:?}",
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
          "brc20swap error handle_add_liq none: ticker_1 not found: {:?}",
          ticker_1.clone().to_string()
        );
        return Err(ExecutionError::ExecutionFailed(BRC20Error::TickerNotFound(
          ticker_1.clone().to_string(),
        )));
      }
    };
    let mut token_1_amt = Brc20Decimal::from_str_with_scale(token_1, ticker_1_info.decimals)
      .map_err(|_e| {
        ExecutionError::ExecutionFailed(BRC20Error::DecimalParseError(token_1.clone()))
      })?;
    if token_1_amt.sign() < 0 || token_1_amt.clone().to_u128().0 > ticker_1_info.total_supply {
      log::debug!(
        "brc20swap error add_liq, token1 amount bigger than total supply: {}, total supply: {}",
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
            "brc20swap error handle_add_liq: pool_pair {:?} not exists",
            pool_pair.clone()
          );
          return Err(ExecutionError::ExecutionFailed(
            BRC20Error::ModuleNotExists(module_id.clone()),
          ));
        }
      };

    let module_info = match context.load_brc20_module_info(&module_id)? {
      Some(module_info) => module_info,
      None => {
        log::debug!(
          "brc20swap error handle_add_liq: module_id {:?} not exists",
          module_id.clone()
        );
        return Err(ExecutionError::ExecutionFailed(
          BRC20Error::ModuleNotExists(module_id.clone()),
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

    // add liq

    let mut first: bool = false;
    let lp_for_pool: Brc20Decimal;
    let lp_for_user: Brc20Decimal;
    if pool_balance.tick_balance[0].sign() == 0 && pool_balance.tick_balance[1].sign() == 0 {
      first = true;
      lp_for_pool = token_0_amt.clone().mul(token_1_amt.clone()).sqrt().unwrap();
      if lp_for_pool < Brc20Decimal::from_u128(1000, 18).unwrap() {
        log::debug!(
          "brc20swap error add_liq, lp less than: {:?}",
          lp_for_pool.clone()
        );
        return Err(ExecutionError::ExecutionFailed(
          BRC20Error::InvalidLpAmount(lp_for_pool.clone()),
        ));
      }
      lp_for_user = lp_for_pool
        .clone()
        .sub(Brc20Decimal::from_u128(1000, 18).unwrap());
    } else {
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
          pool_balance.lp_balance = pool_balance.lp_balance + lp_fee.clone();

          let lp_fee_address = UtxoAddress::from_script(
            ScriptBuf::from_hex(&module_info.lp_fee_pk_script.clone())
              .unwrap()
              .as_script(),
            chain,
          );
          let lp_key =
            BRC20ModuleAddressLPTokenBalanceKey::new(module_id, &lp_fee_address, &pool_pair);
          let mut lp_token_balance = context
            .load_brc20_module_address_lp_token_balance(&lp_key)?
            .unwrap_or(BRC20ModuleLPTokenBalance::new_with_default());
          lp_token_balance.balance = lp_token_balance.balance + lp_fee.clone();
          context.update_brc20_module_address_lp_token_balance(&lp_key, lp_token_balance)?;
        }
      }

      // calculate the amount of liquidity tokens acquired
      let token_1_adjust_amt = pool_balance.tick_balance[token_1_idx]
        .clone()
        .mul(token_0_amt.clone())
        .div(pool_balance.tick_balance[token_0_idx].clone());
      if token_1_amt.cmp(&token_1_adjust_amt) != Ordering::Less {
        token_1_amt = token_1_adjust_amt;
      } else {
        let token_0_adjust_amt = pool_balance.tick_balance[token_0_idx]
          .clone()
          .mul(token_1_amt.clone())
          .div(pool_balance.tick_balance[token_1_idx].clone());
        token_0_amt = token_0_adjust_amt;
      }

      let lp_0 = pool_balance
        .lp_balance
        .clone()
        .mul(token_0_amt.clone())
        .div(pool_balance.tick_balance[token_0_idx].clone());
      let lp_1 = pool_balance
        .lp_balance
        .clone()
        .mul(token_1_amt.clone())
        .div(pool_balance.tick_balance[token_1_idx].clone());

      if lp_0.cmp(&lp_1) == Ordering::Greater {
        lp_for_pool = lp_1;
      } else {
        lp_for_pool = lp_0;
      }
      lp_for_user = lp_for_pool.clone();
    }

    if lp_for_user.cmp(
      &expected_lp_amt
        .clone()
        .mul(
          Brc20Decimal::from_u128(1000, 3)
            .unwrap()
            .sub(slippage_amt.clone()),
        )
        .div(Brc20Decimal::from_u128(1000, 3).unwrap()),
    ) == Ordering::Less
    {
      log::debug!(
        "brc20swap error add_liq: insufficient lp token balance: has: {:?}, wants: {:?}",
        lp_for_user.clone(),
        expected_lp_amt.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InsufficientLPTokenBalance(lp_for_user.clone(), expected_lp_amt.clone()),
      ));
    }

    // user balance check
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

    if token_0_balance.swap_account_balance < token_0_amt.clone() {
      log::debug!(
        "brc20swap error add_liq, token_0 swap account balance insufficient: has: {:?}, wants: {:?}",
        token_0_balance.swap_account_balance.clone(),
        token_0_amt.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InsufficientModuleBalance(
          token_0_balance.swap_account_balance.clone(),
          token_0_amt.clone(),
        ),
      ));
    }
    if token_1_balance.swap_account_balance < token_1_amt.clone() {
      log::debug!(
        "brc20swap error add_liq, token_1 swap account balance insufficient: has: {:?}, wants: {:?}",
        token_1_balance.swap_account_balance.clone(),
        token_1_amt.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InsufficientModuleBalance(
          token_1_balance.swap_account_balance.clone(),
          token_1_amt.clone(),
        ),
      ));
    }

    log::debug!(
      "brc20swap add_liq, update address token balance, {:?} sub {:?}, {:?} sub {:?}",
      ticker_0.to_lowercase(),
      token_0_amt.clone(),
      ticker_1.to_lowercase(),
      token_1_amt.clone()
    );

    token_0_balance.swap_account_balance =
      token_0_balance.swap_account_balance - token_0_amt.clone();
    token_1_balance.swap_account_balance =
      token_1_balance.swap_account_balance - token_1_amt.clone();

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

    // lp balance update
    // lp-user-balance
    let lp_key = BRC20ModuleAddressLPTokenBalanceKey::new(module_id, &sender, &pool_pair);
    let mut lp_balance = context
      .load_brc20_module_address_lp_token_balance(&lp_key)?
      .unwrap_or(BRC20ModuleLPTokenBalance::new_with_default());
    lp_balance.balance = lp_balance.balance.clone() + lp_for_user.clone();

    // zero address lp balance update
    if first {
      let zero_lp_key = BRC20ModuleAddressLPTokenBalanceKey::new(
        module_id,
        &UtxoAddress::from_script(Script::from_bytes(&ZERO_ADDRESS_PKSCRIPT.as_bytes()), chain),
        &pool_pair,
      );
      let mut zero_lp_balance = context
        .load_brc20_module_address_lp_token_balance(&zero_lp_key)?
        .unwrap_or(BRC20ModuleLPTokenBalance::new_with_default());

      zero_lp_balance.balance =
        zero_lp_balance.balance.clone() + Brc20Decimal::from_u128(1000, 18).unwrap();
      context.update_brc20_module_address_lp_token_balance(&zero_lp_key, zero_lp_balance)?;
    }

    log::debug!(
      "brc20swap add_liq, update pool balance, 0: {:?} add {:?}, 1: {:?} add {:?}, lp_balance: {:?} add {:?}",
      pool_balance.tick_balance[token_0_idx].clone(),
      token_0_amt.clone(),
      pool_balance.tick_balance[token_1_idx].clone(),
      token_1_amt.clone(),
      pool_balance.lp_balance,
      lp_for_pool.clone()
    );

    // change in pool balance
    pool_balance.tick_balance[token_0_idx] =
      pool_balance.tick_balance[token_0_idx].clone() + token_0_amt.clone();
    pool_balance.tick_balance[token_1_idx] =
      pool_balance.tick_balance[token_1_idx].clone() + token_1_amt.clone();
    pool_balance.lp_balance = pool_balance.lp_balance + lp_for_pool.clone();

    // update last_root_k
    pool_balance.last_root_k = pool_balance.tick_balance[token_0_idx]
      .clone()
      .mul(pool_balance.tick_balance[token_1_idx].clone())
      .sqrt()
      .unwrap();

    context.update_brc20_module_address_lp_token_balance(&lp_key, lp_balance)?;
    context.update_brc20_module_swap_poolpair_balance(&module_id, &pool_pair, pool_balance)?;

    log::debug!("brc20swap handle_add_liq finished: {:?}", self);
    Ok(())
  }
}

mod test {
  use super::*;

  #[test]
  fn test_add_liq() {
    let chain = Chain::Mainnet;
    let address = UtxoAddress::from_script(
      Script::from_bytes(&ZERO_ADDRESS_PKSCRIPT.as_bytes()),
      &chain,
    );
    println!("address: {:?}", address);
  }
}
