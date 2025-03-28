use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BRC20OpType {
  Deploy,
  Mint,
  InscribeTransfer,
  Transfer,
  CreateModule,
  Withdraw,
  Commit,
  TransferWithdraw,
  TransferCommit,
}

impl From<&BRC20Operation> for BRC20OpType {
  fn from(value: &BRC20Operation) -> Self {
    match value {
      BRC20Operation::Deploy(_) => BRC20OpType::Deploy,
      BRC20Operation::Mint { .. } => BRC20OpType::Mint,
      BRC20Operation::InscribeTransfer{ .. } => BRC20OpType::InscribeTransfer,
      BRC20Operation::Transfer { .. } => BRC20OpType::Transfer,
      BRC20Operation::CreateModule(_) => BRC20OpType::CreateModule,
      BRC20Operation::Withdraw(_) => BRC20OpType::Withdraw,
      BRC20Operation::Commit(_) => BRC20OpType::Commit,
      BRC20Operation::TransferWithdraw(_) => BRC20OpType::TransferWithdraw,
      BRC20Operation::TransferCommit(_) => BRC20OpType::TransferCommit,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum BRC20Event {
  Deploy(DeployEvent),
  Mint(MintEvent),
  InscribeTransfer(InscribeTransferEvent),
  Transfer(TransferEvent),
  CreateModule(CreateModuleEvent),
  InscribeWithdraw(InscribeWithdrawEvent),
  TransferWithdraw(TransferWithdrawEvent),
  InscribeCommit(InscribeCommitEvent),
  TransferCommit(TransferCommitEvent),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeployEvent {
  pub ticker: BRC20Ticker,
  pub total_supply: u128,
  pub decimals: u8,
  pub self_minted: bool,
  pub max_mint_limit: u128,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct InscribeTransferEvent {
  pub ticker: BRC20Ticker,
  pub amount: u128,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MintEvent {
  pub ticker: BRC20Ticker,
  pub amount: u128,
  pub clipped: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TransferEvent {
  pub ticker: BRC20Ticker,
  pub amount: u128,
  pub send_to_coinbase: bool,
  pub burned: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CreateModuleEvent {
  pub name: String,
  pub source: String,
  pub init: CreateModuleInitFields,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CreateModuleInitFields {
  pub swap_fee_rate: String,
  pub gas_tick: String,
  pub gas_to: String,
  pub fee_to: String,
  pub sequencer: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct InscribeWithdrawEvent {
  pub ticker: BRC20Ticker,
  pub amount: String,
  pub module: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TransferWithdrawEvent {
  pub ticker: BRC20Ticker,
  pub amount: String,
  pub module: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct InscribeCommitEvent {
  pub module: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TransferCommitEvent {
  pub module: String,
}
