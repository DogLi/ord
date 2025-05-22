use std::ops::{Add, Div, Mul, Sub};

const DIRECTION_EXACT_IN: &str = "exactIn";
const DIRECTION_EXACT_OUT: &str = "exactOut";

use super::*;
use crate::okx::brc20::{
  brc20_decimal::Brc20Decimal,
  entry::{BRC20ModulePoolPair, BRC20ModuleTokenBalance},
  operation::commit::SwapFunctionData,
};
impl SwapFunctionData {
  pub(super) fn handle_swap(
    &self,
    chain: &Chain,
    module_id: &String,
    context: &mut TableContext,
    fee_rate: Brc20Decimal,
  ) -> Result<(), ExecutionError> {
    log::info!("brc20swap handle_swap: {:?}", self);

    let sender = UtxoAddress::from_str(self.address.as_ref().unwrap(), chain.network())
      .map_err(|e| ExecutionError::ExecutionFailed(BRC20Error::AddressParseError(e.to_string())))?;
    let params = self.params.clone().unwrap();
    if params.len() != 7 {
      log::debug!(
        "brc20swap error handle_swap: invalid params: {:?}, length: {:?}",
        params,
        params.len()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InvalidParamsLength(params.len(), 7),
      ));
    }

    let token_0 = BRC20Ticker::from_str(&params[0])
      .map_err(|e| ExecutionError::ExecutionFailed(BRC20Error::TickerParse(e)))?;
    let token_1 = BRC20Ticker::from_str(&params[1])
      .map_err(|e| ExecutionError::ExecutionFailed(BRC20Error::TickerParse(e)))?;
    let direction = params[4].as_str();
    if direction != DIRECTION_EXACT_IN && direction != DIRECTION_EXACT_OUT {
      log::error!(
        "brc20swap error handle_swap: invalid direction: {:?}",
        direction,
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InvalidSwapDirection(direction.to_string()),
      ));
    }

    let token_in: BRC20Ticker;
    let token_in_amt_str: &String;
    let token_out: BRC20Ticker;
    let token_out_amt_str: &String;
    if direction == DIRECTION_EXACT_IN {
      token_in = BRC20Ticker::from_str(&params[2])
        .map_err(|e| ExecutionError::ExecutionFailed(BRC20Error::TickerParse(e)))?;
      token_in_amt_str = &params[3];

      if token_in == token_0 {
        token_out = token_1.clone();
      } else {
        token_out = token_0.clone();
      }
      token_out_amt_str = &params[5];
    } else {
      token_out = BRC20Ticker::from_str(&params[2])
        .map_err(|e| ExecutionError::ExecutionFailed(BRC20Error::TickerParse(e)))?;
      token_out_amt_str = &params[3];

      if token_out == token_0 {
        token_in = token_1.clone();
      } else {
        token_in = token_0.clone();
      }
      token_in_amt_str = &params[5];
    }

    let token_in_info = match context.load_brc20_ticker_info(&token_in)? {
      Some(ticker_info) => ticker_info,
      None => {
        log::debug!(
          "brc20swap error handle_swap: token_in not found: {}",
          token_in.clone().to_string()
        );
        return Err(ExecutionError::ExecutionFailed(BRC20Error::TickerNotFound(
          token_in.to_string(),
        )));
      }
    };
    let token_in_amt = Brc20Decimal::from_str_with_scale(token_in_amt_str, token_in_info.decimals)
      .map_err(|_e| {
        log::debug!(
          "brc20swap error handle_swap: token_in_amt decimal parse error: {}",
          token_in_amt_str.clone(),
        );
        ExecutionError::ExecutionFailed(BRC20Error::DecimalParseError(token_in_amt_str.clone()))
      })?;
    if token_in_amt.sign() < 0 || token_in_amt.clone().to_u128().0 > token_in_info.total_supply {
      log::debug!(
        "brc20swap error handle_swap: token amount bigger than total supply: {}, total supply: {}",
        token_in_amt.clone(),
        token_in_info.total_supply.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InvalidTickerAmount(token_in_amt.clone().to_u128().0),
      ));
    }
    let token_out_info = match context.load_brc20_ticker_info(&token_out)? {
      Some(ticker_info) => ticker_info,
      None => {
        log::debug!(
          "brc20swap error handle_swap: token_out not found: {}",
          token_out.clone().to_string()
        );
        return Err(ExecutionError::ExecutionFailed(BRC20Error::TickerNotFound(
          token_out.to_string(),
        )));
      }
    };
    let token_out_amt =
      Brc20Decimal::from_str_with_scale(token_out_amt_str, token_out_info.decimals).map_err(
        |_e| {
          log::debug!(
            "brc20swap error handle_swap: token_out_amt decimal parse error: {}",
            token_out_amt_str.clone(),
          );
          ExecutionError::ExecutionFailed(BRC20Error::DecimalParseError(token_out_amt_str.clone()))
        },
      )?;
    if token_out_amt.sign() < 0 || token_out_amt.clone().to_u128().0 > token_out_info.total_supply {
      log::debug!(
        "brc20swap error handle_swap: token amount bigger than total supply: {}, total supply: {}",
        token_out_amt,
        token_out_info.total_supply
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InvalidTickerAmount(token_out_amt.clone().to_u128().0),
      ));
    }

    let pool_pair = BRC20ModulePoolPair::new(&token_0, &token_1);
    let mut pool_balance =
      match context.load_brc20_module_swap_poolpair_balance(&module_id, &pool_pair)? {
        Some(pool) => pool,
        None => {
          log::debug!(
            "brc20swap error handle_swap: pool_pair not found: {}",
            pool_pair.clone().to_string(),
          );
          return Err(ExecutionError::ExecutionFailed(
            BRC20Error::SwapPoolTotalBalanceNotExists(pool_pair.to_string()),
          ));
        }
      };
    // let module_info = match context.load_brc20_module_info(&module_id)? {
    //   Some(module_info) => module_info,
    //   None => {
    //     log::debug!(
    //       "brc20swap error handle_swap: module_id not found: {}",
    //       module_id.clone(),
    //     );
    //     return Err(ExecutionError::ExecutionFailed(
    //       BRC20Error::ModuleNotExists(module_id.clone()),
    //     ));
    //   }
    // };

    // token order
    let token_in_idx: usize;
    let token_out_idx: usize;
    if pool_balance.tick[0] == token_in.clone() {
      token_in_idx = 0;
      token_out_idx = 1;
    } else {
      token_in_idx = 1;
      token_out_idx = 0;
    }

    let slippage_amt = Brc20Decimal::from_str_with_scale(&params[6], 3).map_err(|_e| {
      log::debug!(
        "brc20swap error handle_swap: slippage decimal parse error: {}",
        params[6].clone(),
      );
      ExecutionError::ExecutionFailed(BRC20Error::DecimalParseError(params[6].clone()))
    })?;

    let amount_in: Brc20Decimal;
    let amount_out: Brc20Decimal;
    if direction == DIRECTION_EXACT_IN {
      if fee_rate.sign() > 0 {
        // with fee
        let amount_in_with_fee = token_in_amt
          .clone()
          .mul(Brc20Decimal::from_u128(1000, 3).unwrap() - fee_rate.clone());
        amount_out = pool_balance.tick_balance[token_out_idx]
          .clone()
          .mul(amount_in_with_fee.clone())
          .div(
            pool_balance.tick_balance[token_in_idx]
              .clone()
              .mul(Brc20Decimal::from_u128(1000, 3).unwrap())
              .add(amount_in_with_fee.clone()),
          )
      } else {
        amount_out = pool_balance.tick_balance[token_out_idx]
          .clone()
          .mul(token_in_amt.clone())
          .div(
            pool_balance.tick_balance[token_in_idx]
              .clone()
              .add(token_in_amt.clone()),
          )
      }

      let amount_out_min = token_out_amt
        .clone()
        .mul(Brc20Decimal::from_u128(1000, 3).unwrap())
        .div(
          Brc20Decimal::from_u128(1000, 3)
            .unwrap()
            .add(slippage_amt.clone()),
        );
      if amount_out.clone() < amount_out_min.clone() {
        log::debug!(
          "brc20swap error handle_swap: amount_out < amount_out_min: {:?}, {:?}",
          amount_out.clone(),
          amount_out_min.clone()
        );
        return Err(ExecutionError::ExecutionFailed(BRC20Error::SlippageError()));
      }
      amount_in = token_in_amt.clone();
    } else {
      if fee_rate.sign() > 0 {
        // with fee
        amount_in = pool_balance.tick_balance[token_in_idx]
          .clone()
          .mul(
            token_out_amt
              .clone()
              .mul(Brc20Decimal::from_u128(1000, 3).unwrap()),
          )
          .div(
            pool_balance.tick_balance[token_out_idx]
              .clone()
              .sub(token_out_amt.clone())
              .mul(
                Brc20Decimal::from_u128(1000, 3)
                  .unwrap()
                  .sub(fee_rate.clone()),
              ),
          )
          .add(Brc20Decimal::from_u128(1, token_in_amt.clone().get_precision()).unwrap());
      } else {
        amount_in = pool_balance.tick_balance[token_in_idx]
          .clone()
          .mul(token_out_amt.clone())
          .div(
            pool_balance.tick_balance[token_out_idx]
              .clone()
              .sub(token_out_amt.clone()),
          )
          .add(Brc20Decimal::from_u128(1, token_in_amt.clone().get_precision()).unwrap());
      }
      let amount_in_max = token_in_amt.clone().mul(
        Brc20Decimal::from_u128(1000, 3)
          .unwrap()
          .add(slippage_amt.clone()),
      );
      if amount_in_max.clone() < amount_in.clone() {
        log::debug!(
          "brc20swap error handle_swap: amount_in_max < amount_in: {:?}, {:?}",
          amount_in_max.clone(),
          amount_in.clone()
        );
        return Err(ExecutionError::ExecutionFailed(BRC20Error::SlippageError()));
      }
      amount_out = token_out_amt.clone();
    }

    // check the balance range, prepare to update.
    if pool_balance.tick_balance[token_out_idx].clone() < amount_out.clone() {
      log::debug!(
        "brc20swap error handle_swap: pool token out balance insufficient: has: {:?}, wants: {:?}",
        pool_balance.tick_balance[token_out_idx].clone(),
        amount_out.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InsufficientPoolBalance(
          pool_balance.tick_balance[token_out_idx].clone(),
          amount_out.clone(),
        ),
      ));
    }

    let mut token_in_balance = context
      .load_brc20_module_address_token_balance(&sender, &module_id, &token_in)?
      .unwrap_or(BRC20ModuleTokenBalance::new_with_scale(
        token_in_amt.clone().get_precision(),
      ));
    if token_in_balance.swap_account_balance < token_in_amt.clone() {
      log::debug!(
        "brc20swap error handle_swap: user token in balance insufficient: has: {:?}, wants: {:?}",
        token_in_balance.swap_account_balance.clone(),
        amount_in.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InsufficientUserModuleBalance(
          token_in_balance.swap_account_balance.clone(),
          amount_in.clone(),
        ),
      ));
    }
    // update balance
    // swap sub
    pool_balance.tick_balance[token_out_idx] =
      pool_balance.tick_balance[token_out_idx].clone() - amount_out.clone();
    token_in_balance.swap_account_balance =
      token_in_balance.swap_account_balance.clone() - amount_in.clone();
    context.update_brc20_module_address_token_balance(
      &sender,
      &module_id,
      &token_in,
      token_in_balance,
    )?;

    let mut token_out_balance = context
      .load_brc20_module_address_token_balance(&sender, &module_id, &token_out)?
      .unwrap_or(BRC20ModuleTokenBalance::new_with_scale(
        token_out_amt.clone().get_precision(),
      ));

    // swap add
    pool_balance.tick_balance[token_in_idx] =
      pool_balance.tick_balance[token_in_idx].clone() + amount_in.clone();
    token_out_balance.swap_account_balance =
      token_out_balance.swap_account_balance.clone() + amount_out.clone();
    context.update_brc20_module_address_token_balance(
      &sender,
      &module_id,
      &token_out,
      token_out_balance,
    )?;

    context.update_brc20_module_swap_poolpair_balance(&module_id, &pool_pair, pool_balance)?;

    log::debug!("brc20swap handle_swap finished: {:?}", self);
    Ok(())
  }
}
