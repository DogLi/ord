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
mod remove_liq;
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

pub const SKIP_INVALID_COMMIT_FUNC_FROM_HEIGHT: u32 = 1000000;

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

    // skip invalid commit
    if self.inscription_id.to_string() == "56ae3cc5e5ca64114cd1fb834254c0d196144ed60b821ad1c0511801c5b16283i0"   // invalid commit due to insufficient balance, [INFO] 这个不能删除，会导致回滚，需要写死到代码
      || self.inscription_id.to_string() == "f31d611021a5b26adda9a91ff2d1dacd52dda6a151147820f56ac9f7dae0a902i0" // invalid commit due to invalid parent
      || self.inscription_id.to_string() == "15c0aff274bfeeb4706fa0a85fa666ace8a3f4bc11e8c446188e7a030966760fi0"
    // invalid commit due to invalid parent
      || self.inscription_id.to_string() == "4cd999f20bede0a7cc2a1d69f1370f8cafd2ca73d1f0984da5d112f623da7eb6i0"
    // invalid commit due to invalid parent
      || self.inscription_id.to_string() == "66a8c7b62b0be2d21d2b7918315a97e4fa83e10b3a0bf161f4ac0768cecf4545i0"
    // invalid commit due to invalid parent
    {
      log::info!(
        "brc20swap execute_inscribe_commit skip invalid commit, txid: {:?}, inscription_id: {:?}",
        self.txid,
        self.inscription_id
      );
      return Ok(BRC20Receipt {
        inscription_id: self.inscription_id,
        sequence_number: self.sequence_number,
        inscription_number: self.inscription_number,
        old_satpoint: self.old_satpoint,
        new_satpoint: self.new_satpoint,
        op_type: BRC20OpType::Commit,
        sender: self.sender.clone(),
        receiver: self.sender.clone(),
        result: Ok(BRC20Event::InscribeCommit(event::InscribeCommitEvent {
          module: self.inscription_id.to_string(),
        })),
      });
    }

    match self.pre_verify_inscribe_commit(index, context) {
      Ok(commit) => context.update_commit_info(&self.inscription_id.to_string(), commit.clone())?,
      Err(e) => return Err(e),
    };

    log::info!(
      "brc20swap execute_inscribe_commit finished {:?}, inscription_id: {:?}",
      self.txid,
      self.inscription_id
    );
    Ok(BRC20Receipt {
      inscription_id: self.inscription_id,
      sequence_number: self.sequence_number,
      inscription_number: self.inscription_number,
      old_satpoint: self.old_satpoint,
      new_satpoint: self.new_satpoint,
      op_type: BRC20OpType::Commit,
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
    height: u32,
  ) -> Result<BRC20Receipt, ExecutionError> {
    log::info!(
      "brc20swap execute_transfer_commit: {:?}, inscription_id: {:?}",
      self.txid,
      self.inscription_id
    );

    if self.inscription_id.to_string()
      == "56ae3cc5e5ca64114cd1fb834254c0d196144ed60b821ad1c0511801c5b16283i0"
      || self.inscription_id.to_string()
        == "15c0aff274bfeeb4706fa0a85fa666ace8a3f4bc11e8c446188e7a030966760fi0"
    {
      log::info!(
        "brc20swap execute_transfer_commit skip invalid commit, txid: {:?}, inscription_id: {:?}",
        self.txid,
        self.inscription_id
      );
      return Ok(BRC20Receipt {
        inscription_id: self.inscription_id,
        sequence_number: self.sequence_number,
        inscription_number: self.inscription_number,
        old_satpoint: self.old_satpoint,
        new_satpoint: self.new_satpoint,
        op_type: BRC20OpType::TransferCommit,
        sender: self.sender.clone(),
        receiver: self.sender.clone(),
        result: Ok(BRC20Event::TransferCommit(event::TransferCommitEvent {
          module: self.inscription_id.to_string(),
        })),
      });
    }

    let BRC20Operation::TransferCommit(commit) = &self.operation else {
      log::debug!(
        "brc20swap error execute_transfer_commit unreachable: {:?}, inscription_id: {:?}",
        self.txid,
        self.inscription_id
      );
      unreachable!()
    };

    log::info!("brc20swap execute_transfer_commit commit: {:?}", commit);

    let commit_module_id = commit.module.clone().unwrap_or("".to_owned());

    let receiver = self.receiver.clone().unwrap();
    let script_buf = receiver.to_script(&index.chain());
    let module_id = match get_module_from_script(&script_buf) {
      Ok(module_id) => {
        if module_id != commit_module_id {
          log::debug!(
            "brc20swap error execute_transfer_commit module_id not match: {:?}, inscription_id: {:?}, module_id: {:?}, commit_module_id: {:?}",
            self.txid,
            self.inscription_id,
            module_id,
            commit_module_id
          );
          return Err(ExecutionError::ExecutionFailed(
            BRC20Error::CommitNotSendToModule(self.inscription_id.to_string()),
          ));
        }
        module_id
      }
      Err(_) => {
        log::debug!(
          "brc20swap error execute_transfer_commit module_id not found: {:?}, inscription_id: {:?}",
          self.txid,
          self.inscription_id
        );
        return Err(ExecutionError::ExecutionFailed(
          BRC20Error::CommitNotSendToModule(self.inscription_id.to_string()),
        ));
      }
    };

    let mut module_info = match context.load_brc20_module_info(&module_id) {
      Ok(Some(module_info)) => {
        if module_info.sequencer_pk_script != self.sender.to_script(&index.chain()).to_hex_string()
        {
          log::debug!(
            "brc20swap error execute_transfer_commit module_info not match: {:?}, inscription_id: {:?}, module_info: {:?}, commit_module_id: {:?}",
            self.txid,
            self.inscription_id,
            module_info,
            commit_module_id
          );
          return Err(ExecutionError::ExecutionFailed(
            BRC20Error::ModuleSequencerInvalid(self.inscription_id.to_string()),
          ));
        }
        module_info
      }
      Ok(None) => {
        log::error!(
          "brc20swap error execute_transfer_commit module_info not found: {:?}, inscription_id: {:?}, module_id: {:?}",
          self.txid,
          self.inscription_id,
          module_id
        );
        return Err(ExecutionError::ExecutionFailed(BRC20Error::ModuleInvalid(
          self.inscription_id.to_string(),
        )));
      },
      Err(e) => {
        log::error!(
          "brc20swap error execute_transfer_commit module_info with error {e:?}, tx_id: {:?}, inscription_id: {:?}, module_id: {:?}",
          self.txid,
          self.inscription_id,
          module_id
        );
        return Err(ExecutionError::ExecutionFailed(BRC20Error::ModuleInvalid(
          self.inscription_id.to_string(),
        )));
      }
    };
    let fee_rate_replaced = match module_info.replace_fee_rate_with_event_value(commit.swap_fee_rate.clone()) {
      Ok(fee_rate_replaced) => fee_rate_replaced,
      Err(e) => {
        log::error!("brc20swap error execute_transfer_commit fee_rate_replaced with error: {}, tx_id: {:?}, inscription_id: {:?}, event_swap_fee_rate: {:?}", e, self.txid, self.inscription_id, commit.swap_fee_rate.clone());
        return Err(ExecutionError::ExecutionFailed(BRC20Error::ModuleInvalid(self.inscription_id.to_string())));
      }
    };

    if commit.parent.clone() == module_info.chain_commit_id && module_info.chain_commit_id.is_some()
    {
      log::error!(
        "brc20swap error execute_transfer_commit parent already sattled: tx_id: {:?}, inscription_id: {:?}, parent: {:?}, chain_commit_id: {:?}",
        self.txid,
        self.inscription_id,
        commit.parent.clone(),
        module_info.chain_commit_id.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::CommitParentAlreadySattled(self.inscription_id.to_string()),
      ));
    }
    if commit.parent.clone().unwrap_or_default()
      != module_info.commit_id.clone().unwrap_or_default()
    {
      log::error!(
        "brc20swap error execute_transfer_commit parent invalid: tx_id: {:?}, inscription_id: {:?}, parent: {:?}, commit_id: {:?}",
        self.txid,
        self.inscription_id,
        commit.parent.clone(),
        module_info.commit_id.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::CommitParentInvalid(self.inscription_id.to_string()),
      ));
    }

    let gas_ticker = BRC20Ticker::from_str(&module_info.gas_tick).unwrap();
    let gas_price_amt = match check_ticker_verify(
      context,
      &gas_ticker,
      &commit.gas_price.clone().unwrap(),
    ) {
      Ok(result) => result,
      Err(_) => {
        log::error!(
            "brc20swap error execute_transfer_commit gas_price_amt invaild: tx_id: {:?}, inscription_id: {:?}, gas_ticker: {:?}, gas_price: {:?}",
            self.txid,
            self.inscription_id,
            gas_ticker,
            commit.gas_price.clone().unwrap()
        );
        return Err(ExecutionError::ExecutionFailed(
          BRC20Error::InvalidGasPrice(commit.gas_price.clone().unwrap()),
        ));
      }
    };
    log::info!(
      "brc20swap execute_transfer_commit gas_price_amt: {:?}",
      gas_price_amt,
    );

    if let Some(data) = &commit.data {
      for item in data {
        let address = match UtxoAddress::from_str(
          item.address.clone().unwrap().as_str(),
          index.chain().network(),
        ) {
          Ok(address) => address,
          Err(_) => {
            log::error!(
              "brc20swap error execute_transfer_commit address invalid: {:?}, inscription_id: {:?}, address: {:?}",
              self.txid,
              self.inscription_id,
              item.address.clone().unwrap()
            );
            return Err(ExecutionError::ExecutionFailed(
              BRC20Error::ModuleSequencerInvalid(self.inscription_id.to_string()),
            ));
          }
        };

        if gas_price_amt.sign() > 0 {
          let mut token_balance = match context.load_brc20_module_address_token_balance(
            &address,
            &commit_module_id,
            &gas_ticker,
          ) {
            Ok(Some(token_balance)) => {
              if token_balance.swap_account_balance.clone() < gas_price_amt.clone() {
                log::debug!(
                  "brc20swap error execute_transfer_commit token_balance insufficient: {:?}, inscription_id: {:?}, token_balance: {:?}, gas_price_amt: {:?}",
                  self.txid,
                  self.inscription_id,
                  token_balance.swap_account_balance.clone(),
                  gas_price_amt.clone()
                );
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
            Ok(None)=> {
              log::error!(
                "brc20swap error execute_transfer_commit token_balance not found: {:?}, inscription_id: {:?}, address: {:?}",
                self.txid,
                self.inscription_id,
                item.address.clone().unwrap()
              );
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::TokenBalanceInsufficient(
                  self.inscription_id.to_string(),
                  item.address.clone().unwrap(),
                  module_info.gas_tick,
                ),
              ));
            },
            Err(e) => {
              log::error!("brc20swap error execute_transfer_commit token_balance with error: {e:?}, tx_id: {:?}, inscription_id: {:?}, address:{:?}",
                self.txid, self.inscription_id, item.address
              );
              return Err(ExecutionError::ExecutionFailed(
                BRC20Error::TokenBalanceInsufficient(
                  self.inscription_id.to_string(),
                  item.address.clone().unwrap(),
                  module_info.gas_tick,
                ),
              ));
            }
          };

          let to_gas_address = UtxoAddress::from_script(
            ScriptBuf::from_hex(&module_info.gas_to_pk_script.clone())
              .unwrap()
              .as_script(),
            &index.chain(),
          );
          let mut to_gas_balance = match context.load_brc20_module_address_token_balance(
            &to_gas_address,
            &commit_module_id,
            &gas_ticker,
          ) {
            Ok(Some(option_balance)) => option_balance,
            Ok(None) | Err(_) => {
              BRC20ModuleTokenBalance::new_with_scale(gas_price_amt.get_precision())
            }
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
            &to_gas_address,
            &commit_module_id,
            &gas_ticker,
            to_gas_balance,
          )?;
        }

        if let Some(function) = &item.function {
          let result = match function.as_str() {
            BRC20_SWAP_FUNCTION_DEPLOY_POOL => {
              item.handle_deploy_pool(&module_id.to_string(), context)
            }
            BRC20_SWAP_FUNCTION_ADD_LIQ => {
              item.handle_add_liq(&index.chain(), &module_id.to_string(), context, fee_rate_replaced.clone())
            }
            BRC20_SWAP_FUNCTION_REMOVE_LIQ => {
              item.handle_remove_liq(&index.chain(), &module_id.to_string(), context, fee_rate_replaced.clone())
            }
            BRC20_SWAP_FUNCTION_SWAP => {
              item.handle_swap(&index.chain(), &module_id.to_string(), context, fee_rate_replaced.clone())
            }
            BRC20_SWAP_FUNCTION_SEND => {
              item.handle_send(&index.chain(), &module_id.to_string(), context)
            }
            BRC20_SWAP_FUNCTION_SENDLP => {
              item.handle_send_lp(&index.chain(), &module_id.to_string(), context)
            }
            BRC20_SWAP_FUNCTION_LOCK => {
              item.handle_lock(&index.chain(), &module_id.to_string(), context)
            }
            BRC20_SWAP_FUNCTION_UNLOCK => {
              item.handle_unlock(&index.chain(), &module_id.to_string(), context)
            }
            BRC20_SWAP_FUNCTION_DECREASE_APPROVAL => {
              item.handle_decrease_approval(&index.chain(), &module_id.to_string(), context)
            }
            _ => {
              log::debug!(
                "brc20swap error execute_transfer_commit invalid function: {:?}, inscription_id: {:?}, function: {:?}",
                self.txid,
                self.inscription_id,
                function.to_string()
              );
              if height < SKIP_INVALID_COMMIT_FUNC_FROM_HEIGHT {
                return Err(ExecutionError::ExecutionFailed(
                  BRC20Error::CommitInvalidFunction(
                    self.inscription_id.to_string(),
                    function.to_string(),
                  ),
                ));
              }
              Ok(())
            }
          };
          match result {
            Ok(_) => {}
            Err(e) => {
              log::error!(
                "brc20swap error execute_transfer_commit function failed with error {e:?}: tx_id: {:?}, inscription_id: {:?}, function: {:?}, item: {:?}",
                self.txid,
                self.inscription_id,
                item.function.clone(),
                item.clone()
              );
              if height < SKIP_INVALID_COMMIT_FUNC_FROM_HEIGHT {
                return Err(ExecutionError::ExecutionFailed(
                  BRC20Error::CommitHandleFunctionFailed(self.inscription_id.to_string()),
                ));
              }
            }
          }
        } else {
          log::error!(
            "brc20swap error execute_transfer_commit invalid function, inscription_id: {:?}, function: {:?}",
            self.inscription_id,
            item.function.clone()
          );
          if height < SKIP_INVALID_COMMIT_FUNC_FROM_HEIGHT {
            return Err(ExecutionError::ExecutionFailed(
              BRC20Error::CommitInvalidFunction(self.inscription_id.to_string(), String::new()),
            ));
          }
        }
      }
    }

    // clear commit info
    context.delete_commit_info(&self.inscription_id.to_string())?;

    // update parent to module
    module_info.chain_commit_id = commit.parent.clone();
    module_info.commit_id = Some(inscription_id.to_string());
    context.insert_brc20_module_info(&module_id, module_info)?;

    log::debug!("brc20swap execute_transfer_commit finished: {:?}", self);
    Ok(BRC20Receipt {
      inscription_id: self.inscription_id,
      sequence_number: self.sequence_number,
      inscription_number: self.inscription_number,
      old_satpoint: self.old_satpoint,
      new_satpoint: self.new_satpoint,
      op_type: BRC20OpType::TransferCommit,
      sender: self.sender.clone(),
      receiver: self.sender.clone(),
      result: Ok(BRC20Event::TransferCommit(event::TransferCommitEvent {
        module: self.inscription_id.to_string(),
      })),
    })
  }
}
