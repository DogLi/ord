use super::{
  entry::{BRC20Balance, BRC20Receipt, BRC20TickerInfo},
  error::BRC20Error,
  event::{BRC20Event, BRC20OpType, DeployEvent, InscribeTransferEvent, MintEvent, TransferEvent},
  *,
};

mod commit;
mod create_module;
mod deploy;
mod inscribe_transfer;
mod mint;
mod transfer;
mod withdraw;

pub type BRC20ExecutionMessageValue = [u8];
impl_bincode_dynamic_entry!(BRC20ExecutionMessage, BRC20ExecutionMessageValue);
/// Represents a message used for executing BRC20 operations.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct BRC20ExecutionMessage {
  txid: Txid,
  inscription_id: InscriptionId,
  sequence_number: u32,
  inscription_number: i32,
  old_satpoint: SatPoint,
  new_satpoint: SatPoint,
  sender: UtxoAddress,
  receiver: Option<UtxoAddress>, // no address, if unbound
  operation: BRC20Operation,
}

impl BRC20ExecutionMessage {
  pub(crate) fn new_from_bundle_message(
    value: &BundleMessage,
    context: &mut TableContext,
  ) -> Result<Option<Self>> {
    let build_message = |operation| {
      Ok(Some(Self {
        txid: value.txid,
        inscription_id: value.inscription_id,
        sequence_number: value.sequence_number,
        inscription_number: value.inscription_number,
        old_satpoint: value.old_satpoint,
        new_satpoint: value.new_satpoint,
        sender: value.sender.clone(),
        receiver: value.receiver.clone(),
        operation,
      }))
    };

    match &value.inscription_action {
      InscriptionAction::Created { sub_type, .. } => {
        if let Some(SubType::BRC20(brc20_operation)) = sub_type {
          build_message(brc20_operation.clone())
        } else {
          Ok(None)
        }
      }
      InscriptionAction::Transferred => match Option::<TransferredInscription>::from(value) {
        Some(transferred_inscription) => {
          match transferred_inscription.extract_and_validate_transfer(context) {
            Ok(Some(brc20_operation)) => build_message(brc20_operation),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
          }
        }
        _ => unreachable!(),
      },
    }
  }
}

impl BRC20ExecutionMessage {
  pub fn execute(
    self,
    index: &Index,
    context: &mut TableContext,
    height: u32,
    blocktime: u32,
  ) -> Result<BRC20Receipt> {
    let result = match &self.operation {
      BRC20Operation::Deploy(..) => self.execute_deploy(context, height, blocktime),
      BRC20Operation::Mint { .. } => self.execute_mint(context, height),
      BRC20Operation::InscribeTransfer(_) => self.execute_inscribe_transfer(context),
      BRC20Operation::Transfer { .. } => self.execute_transfer(index, context),
      BRC20Operation::CreateModule(..) => {
        self.execute_create_module(&self.inscription_id, index, context)
      }
      BRC20Operation::Withdraw(..) => self.execute_inscribe_withdraw(context),
      BRC20Operation::Commit(..) => self.execute_inscribe_commit(index, context),
      BRC20Operation::TransferWithdraw(..) => self.execute_transfer_withdraw(index, context),
      BRC20Operation::TransferCommit(..) => {
        self.execute_transfer_commit(&self.inscription_id, index, context, height)
      }
    };

    match result {
      Ok(receipt) => Ok(receipt),
      Err(ExecutionError::ExecutionFailed(e)) => {
        // TODO: remove this after data verification
        // [INFO]: create module 在115422高度遇见source不匹配的事件，是非法的swap操作，而非程序错误，先跳过。铭文ID：8eb007fea05b4459e39a2e62912b264a285f6dd0471b267f00703aeb0a9977d3
        // [INFO]: withdraw 在高度159742高度遇到非法提现，跳过。txid: fc1d944710cabc67ecf6421fc0bcd2820ea2614c249bf23f86aac809339923e0
        //         会遇到invalid withdraw，直接跳过188516高度
        // [INFO]: commit 在高度188355高度遇到非法swap，是第一个func，所以不需要回滚整个commit事件，直接跳过。txid: bc18513f78b1e2926e483c5d728b03d1098a752e12f0be7ff78851e9b55b0920 (inscribe_commit对应tx为56ae3cc5e5ca64114cd1fb834254c0d196144ed60b821ad1c0511801c5b16283)
        //         又碰到了一个非法的commit事件， height:188372，txid: b9dfbfb1dec1764a3870638faf173974ac4ca44e689cb82a1e2ae1d56610c4e8, inscription_id:f31d611021a5b26adda9a91ff2d1dacd52dda6a151147820f56ac9f7dae0a902i0
        //            invalid parent, 因为在188355高度碰到了非法的commit, 直接跳过了，db没有更新module的commit_id, 而188372这个height的commit事件的parent是188355的commit_id, 所以是非法的commit，直接跳到下一个合法的commit高度：189384
        if (matches!(self.operation, BRC20Operation::CreateModule(_)) && height > 115422)
          || matches!(self.operation, BRC20Operation::Commit(_))
          || matches!(self.operation, BRC20Operation::TransferCommit(_))
        {
          panic!("brc20swap operation failed, ExecutionError{:?}", e);
        }
        Ok(BRC20Receipt {
          // Handle specific execution failure
          inscription_id: self.inscription_id,
          sequence_number: self.sequence_number,
          inscription_number: self.inscription_number,
          old_satpoint: self.old_satpoint,
          new_satpoint: self.new_satpoint,
          op_type: BRC20OpType::from(&self.operation),
          sender: self.sender.clone(),
          receiver: self.receiver.unwrap_or(self.sender),
          result: Err(e),
        })
      }
      Err(e) => {
        log::error!(
          "brc20 execution failed: txid = {}, inscription_id = {}, error = {:?}",
          self.txid,
          self.inscription_id,
          e
        );
        // TODO: remove this after data verification
        if matches!(self.operation, BRC20Operation::CreateModule(_))
          || matches!(self.operation, BRC20Operation::Withdraw(_))
          || matches!(self.operation, BRC20Operation::Commit(_))
          || matches!(self.operation, BRC20Operation::TransferWithdraw(_))
          || matches!(self.operation, BRC20Operation::TransferCommit(_))
        {
          panic!("brc20swap operation failed, {:?}", e);
        }
        Err(e.into())
      }
    }
  }
}

#[derive(Debug, thiserror::Error)]
pub(super) enum ExecutionError {
  #[error("Storage error: {0}")]
  Storage(#[from] redb::StorageError),
  #[error("Execution failed: {0}")]
  ExecutionFailed(#[from] BRC20Error),
  #[error("Unexpected error: {0}")]
  Unexpected(#[from] Error),
}
