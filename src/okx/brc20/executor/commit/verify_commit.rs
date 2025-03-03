use super::*;
use crate::okx::{
  brc20::{ticker::BRC20Ticker, verify::check_ticker_verify},
  utils::decode_tokens_from_swap_pair,
};

impl BRC20ExecutionMessage {
  pub(super) fn pre_verify_inscribe_commit(
    &self,
    index: &Index,
    context: &mut TableContext,
  ) -> Result<Commit, ExecutionError> {
    let BRC20Operation::Commit(commit) = &self.operation else {
      unreachable!()
    };

    let module_id = commit.module.clone().unwrap();
    if module_id.to_lowercase() != module_id {
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::ModuleIDInvalid(module_id.clone()),
      ));
    }

    let module_info = context.load_brc20_module_info(&module_id)?;
    if module_info.is_none() {
      return Err(ExecutionError::ExecutionFailed(BRC20Error::ModuleNotFound(
        module_id.clone(),
      )));
    }

    let module_info = module_info.unwrap();
    let gas_tick = BRC20Ticker::from_str(&module_info.gas_tick).unwrap();
    let gas_price = commit.gas_price.clone().unwrap();
    let gas_price_amt = check_ticker_verify(context, &gas_tick, &gas_price);
    if gas_price_amt.is_err() {
      log::error!("Invalid gas price: {}", gas_price);
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InvalidGasPrice(gas_price.clone()),
      ));
    }

    if let Some(data) = &commit.data {
      for f in data.iter() {
        match f.function.as_ref().unwrap().as_str() {
          BRC20_SWAP_FUNCTION_DEPLOY_POOL => {
            let params = f.params.clone().unwrap();
            if params.len() != 2 {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "deploy_pool".to_string(),
                  "params invalid".to_string(),
                ),
              ));
            }

            let token0 = BRC20Ticker::from_str(&params[0]).unwrap();
            let token1 = BRC20Ticker::from_str(&params[1]).unwrap();

            if token0 == token1 {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "deploy_pool".to_string(),
                  "same tokens".to_string(),
                ),
              ));
            }

            if check_ticker_verify(context, &token0, &String::new()).is_err() {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "deploy_pool".to_string(),
                  "tick0 invalid".to_string(),
                ),
              ));
            }

            if check_ticker_verify(context, &token1, &String::new()).is_err() {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "deploy_pool".to_string(),
                  "tick1 invalid".to_string(),
                ),
              ));
            }
          }

          BRC20_SWAP_FUNCTION_ADD_LIQ => {
            if f.params.clone().unwrap().len() != 6 {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "addLiq".to_string(),
                  "params invalid".to_string(),
                ),
              ));
            }

            let params = f.params.clone().unwrap();

            let token0 = BRC20Ticker::from_str(&params[0]).unwrap();
            let token1 = BRC20Ticker::from_str(&params[1]).unwrap();
            let token0_amt = params[2].clone();
            let token1_amt = params[3].clone();
            let token_lp_amt = params[4].clone();
            let slippage = params[5].clone();

            if check_ticker_verify(context, &token0, &token0_amt).is_err() {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction("addLiq".to_string(), "amt0 invalid".to_string()),
              ));
            }

            if check_ticker_verify(context, &token1, &token1_amt).is_err() {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction("addLiq".to_string(), "amt1 invalid".to_string()),
              ));
            }

            if FixedPoint::new_from_str(&token_lp_amt, FixedPoint::MAX_SCALE).is_err() {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "addLiq".to_string(),
                  "amtLp invalid".to_string(),
                ),
              ));
            }

            if FixedPoint::new_from_str(&slippage, 3).is_err() {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "addLiq".to_string(),
                  "slippage invalid".to_string(),
                ),
              ));
            }
          }

          BRC20_SWAP_FUNCTION_REMOVE_LIQ => {
            if f.params.clone().unwrap().len() != 6 {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "removeLiq".to_string(),
                  "params invalid".to_string(),
                ),
              ));
            }

            let params = f.params.clone().unwrap();

            let token0 = BRC20Ticker::from_str(&params[0]).unwrap();
            let token1 = BRC20Ticker::from_str(&params[1]).unwrap();
            let token_lp_amt = params[2].clone();
            let token0_amt = params[3].clone();
            let token1_amt = params[4].clone();
            let slippage = params[5].clone();

            if FixedPoint::new_from_str(&token_lp_amt, FixedPoint::MAX_SCALE).is_err() {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "removeLiq".to_string(),
                  format!("amtLp invalid, {}/{})", token0, token1),
                ),
              ));
            }

            if check_ticker_verify(context, &token0, &token0_amt).is_err() {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "removeLiq".to_string(),
                  "amt0 invalid".to_string(),
                ),
              ));
            }

            if check_ticker_verify(context, &token1, &token1_amt).is_err() {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "removeLiq".to_string(),
                  "amt1 invalid".to_string(),
                ),
              ));
            }

            if FixedPoint::new_from_str(&slippage, 3).is_err() {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "removeLiq".to_string(),
                  "slippage invalid".to_string(),
                ),
              ));
            }
          }

          BRC20_SWAP_FUNCTION_SWAP => {
            let params = f.params.clone().unwrap();
            if params.len() != 7 {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction("swap".to_string(), "params invalid".to_string()),
              ));
            }

            let token0 = &params[0];
            let token1 = &params[1];

            let token = &params[2];
            if token != token0 && token != token1 {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction("swap".to_string(), "token invalid".to_string()),
              ));
            }

            let direction = &params[4];
            let (token_in, token_in_amt, token_out, token_out_amt);
            if direction == "exactIn" {
              token_in = &params[2];
              token_in_amt = &params[3];

              if token_in == token0 {
                token_out = token1;
              } else {
                token_out = token0;
              }
              token_out_amt = &params[5];
            } else if direction == "exactOut" {
              token_out = &params[2];
              token_out_amt = &params[3];

              if token_out == token0 {
                token_in = token1;
              } else {
                token_in = token0;
              }
              token_in_amt = &params[5];
            } else {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "swap".to_string(),
                  "direction invalid".to_string(),
                ),
              ));
            }

            if check_ticker_verify(
              context,
              &BRC20Ticker::from_str(&token_in).unwrap(),
              &token_in_amt,
            )
            .is_err()
            {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "swap".to_string(),
                  "token amount invalid".to_string(),
                ),
              ));
            }

            if check_ticker_verify(
              context,
              &BRC20Ticker::from_str(&token_out).unwrap(),
              &token_out_amt,
            )
            .is_err()
            {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction("swap".to_string(), "amt1 invalid".to_string()),
              ));
            }

            let slippage = params[6].clone();
            if FixedPoint::new_from_str(&slippage, 3).is_err() {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "swap".to_string(),
                  "slippage invalid".to_string(),
                ),
              ));
            }
          }

          BRC20_SWAP_FUNCTION_DECREASE_APPROVAL => {
            let params = f.params.clone().unwrap();
            if params.len() != 2 {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "decreaseApproval".to_string(),
                  "params invalid".to_string(),
                ),
              ));
            }

            let token = &params[0];
            let token_amt = &params[1];

            if check_ticker_verify(context, &BRC20Ticker::from_str(&token).unwrap(), &token_amt)
              .is_err()
            {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "decreaseApproval".to_string(),
                  "amt invalid".to_string(),
                ),
              ));
            }
          }

          BRC20_SWAP_FUNCTION_SEND => {
            let params = f.params.clone().unwrap();
            if params.len() != 3 {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction("send".to_string(), "params invalid".to_string()),
              ));
            }

            let address_to = &params[0];
            if UtxoAddress::get_script_by_str(&address_to.clone(), &index.chain()).is_err() {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction("send".to_string(), "addr invalid".to_string()),
              ));
            }

            let token = &params[1];
            let token_amt = &params[2];

            if check_ticker_verify(context, &BRC20Ticker::from_str(&token).unwrap(), &token_amt)
              .is_err()
            {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction("send".to_string(), "amt invalid".to_string()),
              ));
            }
          }

          BRC20_SWAP_FUNCTION_SENDLP => {
            let params = f.params.clone().unwrap();
            if params.len() != 4 {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "sendlp".to_string(),
                  "params invalid".to_string(),
                ),
              ));
            }

            let address_to = &params[0];
            if UtxoAddress::get_script_by_str(&address_to.clone(), &index.chain()).is_err() {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction("sendlp".to_string(), "addr invalid".to_string()),
              ));
            }

            let token0 = &params[1];
            let token1 = &params[2];

            if check_ticker_verify(
              context,
              &BRC20Ticker::from_str(&token0).unwrap(),
              &String::new(),
            )
            .is_err()
            {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "sendlp".to_string(),
                  "token0 invalid".to_string(),
                ),
              ));
            }

            if check_ticker_verify(
              context,
              &BRC20Ticker::from_str(&token1).unwrap(),
              &String::new(),
            )
            .is_err()
            {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "sendlp".to_string(),
                  "token1 invalid".to_string(),
                ),
              ));
            }

            let pool_pair = format!("{}/{}", token0, token1);
            if decode_tokens_from_swap_pair(&pool_pair).is_err() {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "sendlp".to_string(),
                  "pool pair invalid".to_string(),
                ),
              ));
            }

            let token_amt_str = &params[3];
            if FixedPoint::new_from_str(token_amt_str, 18).is_err() {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  "sendlp".to_string(),
                  format!("amtLp invalid, {}", token_amt_str),
                ),
              ));
            }
          }

          BRC20_SWAP_FUNCTION_LOCK | BRC20_SWAP_FUNCTION_UNLOCK => {
            let params = f.params.clone().unwrap();
            let function = f.function.clone().unwrap();
            if params.len() != 3 {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(function, "params invalid".to_string()),
              ));
            }

            let token0 = &params[0];
            let token1 = &params[1];

            if check_ticker_verify(
              context,
              &BRC20Ticker::from_str(&token0).unwrap(),
              &String::new(),
            )
            .is_err()
            {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(function, "token0 invalid".to_string()),
              ));
            }

            if check_ticker_verify(
              context,
              &BRC20Ticker::from_str(&token1).unwrap(),
              &String::new(),
            )
            .is_err()
            {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(function, "token1 invalid".to_string()),
              ));
            }

            let pool_pair = format!("{}/{}", token0, token1);
            if decode_tokens_from_swap_pair(&pool_pair).is_err() {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(function, "pool pair invalid".to_string()),
              ));
            }

            let token_amt_str = &params[2];
            if FixedPoint::new_from_str(token_amt_str, 18).is_err() {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  function,
                  format!("amtLp invalid, {}", token_amt_str),
                ),
              ));
            }
          }

          _ => {
            let function = f.function.clone().unwrap();
            log::warn!(
              "commit[{}] invalid function: {}, id: {}",
              self.txid,
              function,
              self.inscription_id
            );
            return Err(ExecutionError::ExecutionFailed(
              BRC20Error::CommitInvalidFunction(function, "function invalid".to_string()),
            ));
          }
        }
      }
    }

    return Ok(commit.clone());
  }
}
