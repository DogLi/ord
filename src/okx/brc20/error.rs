use super::{brc20_decimal::Brc20Decimal, *};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, thiserror::Error, Deserialize, Serialize)]
pub enum BRC20Error {
  #[error("Database error: {0}")]
  DBError(String),

  #[error("Failed to parse ticker: {0}")]
  TickerParse(#[from] ticker::Error),

  #[error("Duplicate deployment detected for ticker: {0}")]
  DuplicateDeployment(String),

  #[error("Ticker '{0}' not found")]
  TickerNotFound(String),

  #[error("Decimals value {0} exceeds the maximum allowed limit of 18")]
  DecimalsExceedLimit(FixedPoint),

  #[error("Ticker has an invalid supply: {0}")]
  InvalidSupply(FixedPoint),

  #[error("Ticker has an invalid max mint limit: {0}")]
  InvalidMaxMintLimit(FixedPoint),

  #[error("Mint amount exceeds the allowed limit: {0}")]
  MintAmountExceedLimit(FixedPoint),

  #[error("Ticker has an invalid amount: {0}")]
  InvalidAmount(FixedPoint),

  #[error("Invalid amount format: {0}")]
  InvalidAmountFormat(String),

  #[error("Invalid address")]
  InvalidAddress(),

  #[error("Minting has reached the maximum supply limit")]
  MintingLimitReached,

  #[error("Insufficient balance: {0} {1}")]
  InsufficientBalance(FixedPoint, FixedPoint),

  #[error("Self-mint operation denied: insufficient permissions")]
  SelfMintPermissionDenied,

  #[error("Numeric error occurred: {0}")]
  NumericError(#[from] fixed_point::NumParseError),

  #[error("Decimal error occurred: {0}")]
  DecimalParseError(String),

  #[error("Address parse error: {0}")]
  AddressParseError(String),

  #[error("Module ID invalid: {0}")]
  ModuleIDInvalid(String),

  #[error("Module not exists: {0}")]
  ModuleNotExists(String),

  #[error("Module source '{0}' not match")]
  ModuleSourceNotMatch(String),

  #[error("Module not found: {0}")]
  ModuleNotFound(String),

  #[error("Module already exists: {0}")]
  ModuleAlreadyExists(String),

  #[error("Invalid ticker amount: {0}")]
  InvalidTickerAmount(u128),

  #[error("Insufficient module balance: {0}, operation balance: {1}")]
  InsufficientModuleBalance(Brc20Decimal, Brc20Decimal),

  #[error("Invalid swap fee rate: {0}")]
  InvalidSwapFeeRate(String),

  #[error("Invalid gas tick: {0}")]
  InvalidGasTick(String),

  #[error("Invalid gas price: {0}")]
  InvalidGasPrice(String),

  #[error("Invalid sequencer: {0}")]
  InvalidSequencer(String),

  #[error("Invalid gas to: {0}")]
  InvalidGasTo(String),

  #[error("Invalid fee to: {0}")]
  InvalidFeeTo(String),

  #[error("Ticker invalid: {0}")]
  TickerInvalid(String),

  #[error("Module balance not exists: module: {0}, ticker: {1}")]
  ModuleBalanceNotExists(String, String),

  #[error("Invalid withdraw amount: {0}")]
  InvalidWithdrawAmount(Brc20Decimal),

  #[error("Insufficient withdraw amount: {0}, available: {1}")]
  InsufficientWithdrawAmount(Brc20Decimal, Brc20Decimal),

  #[error("Inscribe withdraw not found: {0}")]
  InscribeWithdrawNotFound(String),

  #[error("Remove inscribe withdraw error: {0}")]
  RemoveInscribeWithdrawError(String),

  #[error("Commit invalid function {0} '{1}'")]
  CommitInvalidFunction(String, String),

  #[error("Commit handle function failed {0}")]
  CommitHandleFunctionFailed(String),

  #[error("Module swap LP token balance not exists, module_id: {0}, pool_pair: {1}")]
  LPTokenBalanceNotExists(String, String),

  #[error("Insufficient LP token balance, has: {0}, wants: {1}")]
  InsufficientLPTokenBalance(Brc20Decimal, Brc20Decimal),

  #[error("Insufficient token balance, has: {0}, wants: {1}")]
  InsufficientPoolBalance(Brc20Decimal, Brc20Decimal),

  #[error("Insufficient user module balance, has: {0}, wants: {1}")]
  InsufficientUserModuleBalance(Brc20Decimal, Brc20Decimal),

  #[error("Swap pool total balance not exists, pool_pair: {0}")]
  SwapPoolTotalBalanceNotExists(String),

  #[error("Duplicate deployment pool pair: {0}")]
  DuplicateDeployPool(String),

  #[error("Module Invalid: {0}")]
  ModuleInvalid(String),

  #[error("Commit not send to module: {0}")]
  CommitNotSendToModule(String),

  #[error("Module sequencer invalid: {0}")]
  ModuleSequencerInvalid(String),

  #[error("Commit address invalid: {0}")]
  CommitAddressInvalid(String),

  #[error("Token balance insufficient: {0} {1} {2}")]
  TokenBalanceInsufficient(String, String, String),

  #[error("Commit missing parent: {0}")]
  CommitMissingParent(String),

  #[error("Commit parent already sattled: {0}")]
  CommitParentAlreadySattled(String),

  #[error("Commit parent invalid: {0}")]
  CommitParentInvalid(String),

  #[error("Invalid LP amount: {0}")]
  InvalidLpAmount(Brc20Decimal),

  #[error("Invalid params length: {0}, expected: {1}")]
  InvalidParamsLength(usize, usize),

  #[error("Over slippage")]
  OverSlippage(),

  #[error("Slippage error")]
  SlippageError(),

  #[error("Invalid swap direction: {0}")]
  InvalidSwapDirection(String),
}
