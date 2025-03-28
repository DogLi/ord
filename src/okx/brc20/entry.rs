use super::{brc20_decimal::Brc20Decimal, *};
use event::{BRC20Event, BRC20OpType};

pub type BRC20BalanceValue = [u8];
impl_bincode_dynamic_entry!(BRC20Balance, BRC20BalanceValue);
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BRC20Balance {
  pub ticker: BRC20Ticker,
  pub total: u128,
  pub available: u128,
}

impl BRC20Balance {
  pub fn new_with_ticker(ticker: &BRC20Ticker) -> Self {
    Self {
      ticker: ticker.clone(),
      total: 0,
      available: 0,
    }
  }
}

pub type BRC20TickerValue = [u8];
impl_bincode_dynamic_entry!(BRC20Ticker, BRC20TickerValue);

pub type BRC20LowerCaseTickerValue = [u8];
impl_bincode_dynamic_entry!(BRC20LowerCaseTicker, BRC20LowerCaseTickerValue);

pub(crate) type BRC20TickerInfoValue = [u8];
impl_bincode_dynamic_entry!(BRC20TickerInfo, BRC20TickerInfoValue);
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BRC20TickerInfo {
  pub ticker: BRC20Ticker,
  pub sequence_number: u32,
  pub inscription_number: i32,
  pub inscription_id: InscriptionId,
  pub total_supply: u128,
  pub burned: u128,
  pub minted: u128,
  pub max_mint_limit: u128,
  pub decimals: u8,
  pub deployer: UtxoAddress,
  pub self_minted: bool,
  pub deployed_block_height: u32,
  pub deployed_timestamp: u32,
  pub latest_minted_block_height: u32,
}

pub(crate) type BRC20TransferAssetValue = [u8];
impl_bincode_dynamic_entry!(BRC20TransferAsset, BRC20TransferAssetValue);
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BRC20TransferAsset {
  pub ticker: BRC20Ticker,
  pub amount: u128,
  pub owner: UtxoAddress,
  pub sequence_number: u32,
  pub inscription_number: i32,
  pub inscription_id: InscriptionId,
}

pub(crate) type BRC20ReceiptsValue = [u8];
impl_bincode_dynamic_entry!(Vec<BRC20Receipt>, BRC20ReceiptsValue);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BRC20Receipt {
  pub inscription_id: InscriptionId,
  pub sequence_number: u32,
  pub inscription_number: i32,
  pub old_satpoint: SatPoint,
  pub new_satpoint: SatPoint,
  pub op_type: BRC20OpType,
  pub sender: UtxoAddress,
  pub receiver: UtxoAddress,
  pub result: Result<BRC20Event, BRC20Error>,
}

pub struct BRC20SwapInfo {
  pub total_module_count: u64,
  pub total_module_address_count: u64,
  pub total_module_swap_pool_address_count: u64,
  pub total_module_swap_pool_pair_count: u64,
  pub all_modules: Vec<BRC20ModuleInfo>,
}

pub type BRC20ModuleInfoValue = [u8];
impl_bincode_dynamic_entry!(BRC20ModuleInfo, BRC20ModuleInfoValue);
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BRC20ModuleInfo {
  pub id: String,
  pub name: String,
  pub deployer_pk_script: String,
  pub sequencer_pk_script: String,
  pub gas_to_pk_script: String,
  pub lp_fee_pk_script: String,

  pub fee_rate_swap: Brc20Decimal,
  pub gas_tick: String,

  pub chain_commit_id: Option<String>,
  pub commit_id: Option<String>,
}

pub type BRC20ModuleAddressTokenBalanceKeyValue = [u8];
impl_bincode_dynamic_entry!(
  BRC20ModuleAddressTokenBalanceKey,
  BRC20ModuleAddressTokenBalanceKeyValue
);
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BRC20ModuleAddressTokenBalanceKey {
  pub address: UtxoAddress,
  pub module_id: String,
  pub ticker: BRC20LowerCaseTicker,
}

pub type BRC20ModuleTokenBalanceValue = [u8];
impl_bincode_dynamic_entry!(BRC20ModuleTokenBalance, BRC20ModuleTokenBalanceValue);
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BRC20ModuleTokenBalance {
  pub swap_account_balance: Brc20Decimal,
  pub available_balance: Brc20Decimal,
  pub pending_withdrawal_amount: Brc20Decimal,
}

impl BRC20ModuleTokenBalance {
  pub fn new_with_scale(scale: u8) -> Self {
    Self {
      swap_account_balance: Brc20Decimal::from_u128(0, scale).unwrap(),
      available_balance: Brc20Decimal::from_u128(0, scale).unwrap(),
      pending_withdrawal_amount: Brc20Decimal::from_u128(0, scale).unwrap(),
    }
  }
}

pub type BRC20ModuleSwapPoolPairBalanceKeyValue = [u8];
impl_bincode_dynamic_entry!(
  BRC20ModuleSwapPoolPairBalanceKey,
  BRC20ModuleSwapPoolPairBalanceKeyValue
);
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BRC20ModuleSwapPoolPairBalanceKey {
  pub module_id: String,
  pub pool_pair: BRC20ModulePoolPair,
}

impl BRC20ModuleSwapPoolPairBalanceKey {
  pub fn new(module_id: &String, pool_pair: &BRC20ModulePoolPair) -> Self {
    Self {
      module_id: module_id.clone(),
      pool_pair: pool_pair.clone(),
    }
  }
}

pub type BRC20ModulePoolPairValue = [u8];
impl_bincode_dynamic_entry!(BRC20ModulePoolPair, BRC20ModulePoolPairValue);
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BRC20ModulePoolPair(Box<[u8]>);
impl BRC20ModulePoolPair {
  pub fn new(token0: &BRC20Ticker, token1: &BRC20Ticker) -> Self {
    let lower_case_token0 = token0.to_lowercase();
    let lower_case_token1 = token1.to_lowercase();

    let (first, second) = if lower_case_token0.to_string() >= lower_case_token1.to_string() {
      (lower_case_token1, lower_case_token0)
    } else {
      (lower_case_token0, lower_case_token1)
    };

    let mut result = Vec::new();
    let first_len = first.len() as u8;
    result.push(first_len);
    result.extend_from_slice(first.to_box().as_ref());
    result.extend_from_slice(second.to_box().as_ref());

    BRC20ModulePoolPair(result.into_boxed_slice())
  }

  pub fn to_string(&self) -> String {
    if self.0.is_empty() {
      return String::new();
    }

    if self.0.len() > 1 {
      return String::from_utf8_lossy(&self.0[1..]).to_string();
    }

    String::new()
  }
}

impl Display for BRC20ModulePoolPair {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", std::str::from_utf8(&self.0).unwrap())
  }
}

pub type BRC20ModuleSwapPoolPairBalanceValue = [u8];
impl_bincode_dynamic_entry!(
  BRC20ModuleSwapPoolPairBalance,
  BRC20ModuleSwapPoolPairBalanceValue
);
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BRC20ModuleSwapPoolPairBalance {
  pub tick: [BRC20Ticker; 2],
  pub tick_balance: [Brc20Decimal; 2],
  pub lp_balance: Brc20Decimal,
  pub last_root_k: Brc20Decimal,
}

impl BRC20ModuleSwapPoolPairBalance {
  pub fn new_with_balance(
    ticker1: BRC20Ticker,
    ticker2: BRC20Ticker,
    ticker1_balance: Brc20Decimal,
    ticker2_balance: Brc20Decimal,
  ) -> Self {
    Self {
      tick: [ticker1, ticker2],
      tick_balance: [ticker1_balance, ticker2_balance],
      lp_balance: Brc20Decimal::from_u128(0, 18).unwrap(),
      last_root_k: Brc20Decimal::from_u128(0, 18).unwrap(),
    }
  }
}

// LP Token Balance
pub type BRC20ModuleAddressLPTokenBalanceKeyValue = [u8];
impl_bincode_dynamic_entry!(
  BRC20ModuleAddressLPTokenBalanceKey,
  BRC20ModuleAddressLPTokenBalanceKeyValue
);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BRC20ModuleAddressLPTokenBalanceKey {
  pub module_id: String,
  pub address: UtxoAddress,
  pub pool_pair: BRC20ModulePoolPair,
}

impl BRC20ModuleAddressLPTokenBalanceKey {
  pub fn new(module_id: &String, address: &UtxoAddress, pool_pair: &BRC20ModulePoolPair) -> Self {
    Self {
      module_id: module_id.clone(),
      address: address.clone(),
      pool_pair: pool_pair.clone(),
    }
  }
}

pub type BRC20ModuleLPTokenBalanceValue = [u8];
impl_bincode_dynamic_entry!(BRC20ModuleLPTokenBalance, BRC20ModuleLPTokenBalanceValue);
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BRC20ModuleLPTokenBalance {
  pub balance: Brc20Decimal,
  pub locked_balance: Brc20Decimal,
}

impl BRC20ModuleLPTokenBalance {
  pub fn new(balance: Brc20Decimal, locked_balance: Brc20Decimal) -> Self {
    Self {
      balance,
      locked_balance,
    }
  }

  pub fn new_with_default() -> Self {
    Self {
      balance: Brc20Decimal::from_u128(0, 18).unwrap(),
      locked_balance: Brc20Decimal::from_u128(0, 18).unwrap(),
    }
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use crate::define_table;
  use redb::{ReadableTable, TableDefinition};
  use tempfile::NamedTempFile;

  #[test]
  fn test_store() {
    let ticker = BRC20Ticker::from_str("aBcD").unwrap();
    let value = ticker.clone().store();
    assert_eq!(value.to_vec(), vec![4, 0, 0, 0, 0, 0, 0, 0, 97, 66, 99, 68]);
    let lower_ticker = ticker.to_lowercase();
    let value = lower_ticker.store();
    assert_eq!(
      value.to_vec(),
      vec![4, 0, 0, 0, 0, 0, 0, 0, 97, 98, 99, 100]
    );
  }

  #[test]
  fn test_load() {
    let value = vec![4, 0, 0, 0, 0, 0, 0, 0, 97, 66, 99, 68];
    let ticker = BRC20Ticker::load(&value);
    assert_eq!(ticker, BRC20Ticker::from_str("aBcD").unwrap());
    let value = vec![4, 0, 0, 0, 0, 0, 0, 0, 97, 98, 99, 100];
    let lower_ticker = BRC20LowerCaseTicker::load(&value);
    assert_eq!(
      lower_ticker,
      BRC20Ticker::from_str("abcd").unwrap().to_lowercase()
    );
  }

  #[test]
  fn test_lower_case_ticker_as_key() {
    define_table!(LOWER_CASE_TICKER, &BRC20LowerCaseTickerValue, u32);

    let db_file = NamedTempFile::new().unwrap();
    let database = redb::Database::builder().create(db_file.path()).unwrap();
    let wtx = database.begin_write().unwrap();
    let mut table = wtx.open_table(LOWER_CASE_TICKER).unwrap();

    let lower_case_ticker = BRC20Ticker::from_str("keys").unwrap().to_lowercase();

    table
      .insert(lower_case_ticker.clone().store().as_ref(), 1)
      .unwrap();

    let retrieved_value = table
      .get(lower_case_ticker.store().as_ref())
      .unwrap()
      .unwrap()
      .value();
    assert_eq!(retrieved_value, 1);
  }

  #[test]
  fn test_lower_case_ticker_as_value() {
    define_table!(LOWER_CASE_TICKER, u32, &BRC20LowerCaseTickerValue);

    let db_file = NamedTempFile::new().unwrap();
    let database = redb::Database::builder().create(db_file.path()).unwrap();
    let wtx = database.begin_write().unwrap();
    let mut table = wtx.open_table(LOWER_CASE_TICKER).unwrap();

    let lower_case_ticker = BRC20Ticker::from_str("value").unwrap().to_lowercase();

    table
      .insert(1, lower_case_ticker.clone().store().as_ref())
      .unwrap();

    let retrieved_value = table
      .get(1)
      .unwrap()
      .map(|v| BRC20LowerCaseTicker::load(v.value()))
      .unwrap();
    assert_eq!(retrieved_value, lower_case_ticker);
  }

  #[test]
  fn test_ticker() {
    define_table!(TICKER, u32, &BRC20TickerValue);

    let db_file = NamedTempFile::new().unwrap();
    let database = redb::Database::builder().create(db_file.path()).unwrap();
    let wtx = database.begin_write().unwrap();
    let mut table = wtx.open_table(TICKER).unwrap();

    let ticker = BRC20Ticker::from_str("valuefractal").unwrap();

    table.insert(1, ticker.clone().store().as_ref()).unwrap();

    let retrieved_value = table
      .get(1)
      .unwrap()
      .map(|v| BRC20Ticker::load(v.value()))
      .unwrap();

    assert_eq!(retrieved_value, ticker);
  }

  #[test]
  fn test_module_info() {
    let module_info = BRC20ModuleInfo {
      id: "ID".to_string(),
      name: "swap".to_string(),
      deployer_pk_script: "bc1qtaf86fqf9hv7r76927fjpxc0mpvedgyp7zjneu".to_string(),
      sequencer_pk_script: "bc1qtaf86fqf9hv7r76927fjpxc0mpvedgyp7zjneu".to_string(),
      gas_to_pk_script: "bc1qam880mjcygnkjny5km39vut89vsnq7yun4nr73".to_string(),
      lp_fee_pk_script: "bc1qaejzncrr87pr7azn9fadwa79cp42acsxeygert".to_string(),
      fee_rate_swap: Brc20Decimal::from_u128(119813216873524654, 12).unwrap(),
      gas_tick: "bSATS_".to_string(),
      chain_commit_id: Some("chain_commit_id".to_string()),
      commit_id: Some("commit_id".to_string()),
    };

    let value = module_info.store();
    assert_eq!(module_info, BRC20ModuleInfo::load(&value));
  }
}
