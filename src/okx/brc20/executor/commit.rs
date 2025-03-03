use std::{
  cmp::Ordering,
  ops::{Add, Sub},
};

use crate::okx::{
  brc20::{entry::BRC20ModuleTokenBalance, verify::check_ticker_verify},
  utils::get_module_from_script,
};

use super::*;

mod add_liq;
mod decrease_approval;
mod deploy_pool;
mod lock;
mod remove_lip;
mod send;
mod sendlp;
mod swap;
mod unlock;
mod verify_commit;

pub const BRC20_SWAP_FUNCTION_DEPLOY_POOL: &str = "deployPool";
pub const BRC20_SWAP_FUNCTION_ADD_LIQ: &str = "addLiq";
pub const BRC20_SWAP_FUNCTION_REMOVE_LIQ: &str = "removeLiq";
pub const BRC20_SWAP_FUNCTION_SWAP: &str = "swap";
pub const BRC20_SWAP_FUNCTION_SEND: &str = "send";
pub const BRC20_SWAP_FUNCTION_SENDLP: &str = "sendLp";
pub const BRC20_SWAP_FUNCTION_LOCK: &str = "lock";
pub const BRC20_SWAP_FUNCTION_UNLOCK: &str = "unlock";
pub const BRC20_SWAP_FUNCTION_DECREASE_APPROVAL: &str = "decreaseApproval";

pub const ZERO_ADDRESS_PKSCRIPT: &str = "\x6a\x20\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";

impl BRC20ExecutionMessage {
  pub(super) fn execute_inscribe_commit(
    &self,
    index: &Index,
    context: &mut TableContext,
  ) -> Result<BRC20Receipt, ExecutionError> {
    log::info!(
      "brc20swap execute_inscribe_commit: {:?}, inscription_id: {:?}",
      self.txid,
      self.inscription_id
    );

    match self.pre_verify_inscribe_commit(index, context) {
      Ok(commit) => context.update_commit_info(&self.inscription_id.to_string(), commit.clone())?,
      Err(e) => return Err(e),
    };

    Ok(BRC20Receipt {
      inscription_id: self.inscription_id,
      sequence_number: self.sequence_number,
      inscription_number: self.inscription_number,
      old_satpoint: self.old_satpoint,
      new_satpoint: self.new_satpoint,
      op_type: BRC20OpType::Withdraw,
      sender: self.sender.clone(),
      receiver: self
        .receiver
        .as_ref()
        .cloned()
        .unwrap_or_else(|| self.sender.clone()),
      result: Ok(BRC20Event::InscribeCommit(event::InscribeCommitEvent {
        module: self.inscription_id.to_string(),
      })),
    })
  }

  pub(super) fn execute_transfer_commit(
    &self,
    inscription_id: &InscriptionId,
    index: &Index,
    context: &mut TableContext,
  ) -> Result<BRC20Receipt, ExecutionError> {
    log::info!(
      "brc20swap execute_transfer_commit: {:?}, inscription_id: {:?}",
      self.txid,
      self.inscription_id
    );

    let BRC20Operation::TransferCommit(commit) = &self.operation else {
      log::debug!(
        "brc20swap execute_transfer_commit unreachable: {:?}, inscription_id: {:?}",
        self.txid,
        self.inscription_id
      );
      unreachable!()
    };
    let commit_module_id = commit.module.clone().unwrap_or("".to_owned());

    let receiver = self.receiver.clone().unwrap();
    let script_buf = receiver.to_script(&index.chain());
    let module_id = match get_module_from_script(&script_buf) {
      Ok(module_id) => {
        if module_id != commit_module_id {
          return Err(ExecutionError::ExecutionFailed(
            BRC20Error::CommitNotSendToModule(self.inscription_id.to_string()),
          ));
        }
        module_id
      }
      Err(_) => {
        return Err(ExecutionError::ExecutionFailed(
          BRC20Error::CommitNotSendToModule(self.inscription_id.to_string()),
        ))
      }
    };

    let mut module_info = match context.load_brc20_module_info(&module_id) {
      Ok(Some(module_info)) => {
        if module_info.sequencer_pk_script != self.sender.to_script(&index.chain()).to_string() {
          return Err(ExecutionError::ExecutionFailed(
            BRC20Error::ModuleSequencerInvalid(self.inscription_id.to_string()),
          ));
        }
        module_info
      }
      Ok(None) | Err(_) => {
        return Err(ExecutionError::ExecutionFailed(BRC20Error::ModuleInvalid(
          self.inscription_id.to_string(),
        )))
      }
    };

    if commit.parent.clone() == module_info.chain_commit_id && module_info.chain_commit_id.is_some()
    {
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::CommitParentAlreadySattled(self.inscription_id.to_string()),
      ));
    }
    if commit.parent.clone() != module_info.commit_id {
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::CommitParentInvalid(self.inscription_id.to_string()),
      ));
    }

    let gas_ticker = BRC20Ticker::from_str(&module_info.gas_tick).unwrap();
    let gas_price_amt =
      match check_ticker_verify(context, &gas_ticker, &commit.gas_price.clone().unwrap()) {
        Ok(result) => brc20_decimal::Brc20Decimal::from_fix_point(result),
        Err(_) => brc20_decimal::Brc20Decimal::from_u128(0, 0).unwrap(),
      };

    if let Some(data) = &commit.data {
      for item in data {
        let address = match UtxoAddress::from_str(
          item.address.clone().unwrap().as_str(),
          index.chain().network(),
        ) {
          Ok(address) => address,
          Err(_) => {
            return Err(ExecutionError::ExecutionFailed(
              BRC20Error::ModuleSequencerInvalid(self.inscription_id.to_string()),
            ))
          }
        };

        if gas_price_amt.sign() > 0 {
          let mut token_balance = match context.load_brc20_module_address_token_balance(
            &address,
            &commit_module_id,
            &gas_ticker,
          ) {
            Ok(Some(token_balance)) => {
              if token_balance.swap_account_balance.cmp(&gas_price_amt) != Ordering::Greater {
                return Err(ExecutionError::ExecutionFailed(
                  BRC20Error::TokenBalanceInsufficient(
                    self.inscription_id.to_string(),
                    item.address.clone().unwrap(),
                    module_info.gas_tick,
                  ),
                ));
              }
              token_balance
            }
            Ok(None) | Err(_) => {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::TokenBalanceInsufficient(
                  self.inscription_id.to_string(),
                  item.address.clone().unwrap(),
                  module_info.gas_tick,
                ),
              ))
            }
          };

          let mut to_gas_balance = match context.load_brc20_module_address_token_balance(
            &UtxoAddress::from_str(
              &module_info.gas_to_pk_script.clone().as_str(),
              index.chain().network(),
            )
            .unwrap(),
            &commit_module_id,
            &gas_ticker,
          ) {
            Ok(Some(option_balance)) => option_balance,
            Ok(None) | Err(_) => BRC20ModuleTokenBalance::new(),
          };

          token_balance.swap_account_balance = token_balance
            .swap_account_balance
            .sub(gas_price_amt.clone());
          to_gas_balance.swap_account_balance = to_gas_balance
            .swap_account_balance
            .add(gas_price_amt.clone());

          context.update_brc20_module_address_token_balance(
            &address,
            &commit_module_id,
            &gas_ticker,
            token_balance,
          )?;

          context.update_brc20_module_address_token_balance(
            &UtxoAddress::from_str(
              &module_info.gas_to_pk_script.clone().as_str(),
              index.chain().network(),
            )
            .unwrap(),
            &commit_module_id,
            &gas_ticker,
            to_gas_balance,
          )?;
        }

        if let Some(function) = &item.function {
          let result = match function.as_str() {
            BRC20_SWAP_FUNCTION_DEPLOY_POOL => {
              item.handle_deploy_pool(&self.inscription_id.to_string(), context)
            }
            BRC20_SWAP_FUNCTION_ADD_LIQ => {
              item.handle_add_liq(&index.chain(), &self.inscription_id.to_string(), context)
            }
            BRC20_SWAP_FUNCTION_REMOVE_LIQ => {
              item.handle_remove_liq(&index.chain(), &self.inscription_id.to_string(), context)
            }
            BRC20_SWAP_FUNCTION_SWAP => {
              item.handle_swap(&index.chain(), &self.inscription_id.to_string(), context)
            }
            BRC20_SWAP_FUNCTION_SEND => {
              item.handle_send(&index.chain(), &self.inscription_id.to_string(), context)
            }
            BRC20_SWAP_FUNCTION_SENDLP => {
              item.handle_send_lp(&index.chain(), &self.inscription_id.to_string(), context)
            }
            BRC20_SWAP_FUNCTION_LOCK => {
              item.handle_lock(&index.chain(), &self.inscription_id.to_string(), context)
            }
            BRC20_SWAP_FUNCTION_UNLOCK => {
              item.handle_unlock(&index.chain(), &self.inscription_id.to_string(), context)
            }
            BRC20_SWAP_FUNCTION_DECREASE_APPROVAL => {
              item.handle_decrease_approval(&self.sender, &self.inscription_id.to_string(), context)
            }
            _ => {
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::CommitInvalidFunction(
                  self.inscription_id.to_string(),
                  function.to_string(),
                ),
              ));
            }
          };
          result?;
        } else {
          return Err(ExecutionError::ExecutionFailed(
            BRC20Error::CommitInvalidFunction(self.inscription_id.to_string(), String::new()),
          ));
        }
      }
    }

    // clear commit info
    context.delete_commit_info(&self.inscription_id.to_string())?;

    // update parent to module
    module_info.chain_commit_id = commit.parent.clone();
    module_info.commit_id = Some(inscription_id.to_string());
    context.insert_brc20_module_info(&module_id, module_info)?;

    Ok(BRC20Receipt {
      inscription_id: self.inscription_id,
      sequence_number: self.sequence_number,
      inscription_number: self.inscription_number,
      old_satpoint: self.old_satpoint,
      new_satpoint: self.new_satpoint,
      op_type: BRC20OpType::Commit,
      sender: self.sender.clone(),
      receiver: self.sender.clone(),
      result: Ok(BRC20Event::TransferCommit(event::TransferCommitEvent {
        module: self.inscription_id.to_string(),
      })),
    })
  }
}
