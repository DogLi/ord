use crate::impl_bincode_dynamic_entry;
use crate::okx::entry::DynamicEntry;
use serde::{Deserialize, Serialize};

pub type WithdrawValue = [u8];
impl_bincode_dynamic_entry!(Withdraw, WithdrawValue);

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Withdraw {
  #[serde(rename = "p")]
  pub p: String,
  #[serde(rename = "op")]
  pub op: String,
  #[serde(rename = "tick")]
  pub tick: String,
  #[serde(rename = "amt")]
  pub amt: String,
  #[serde(rename = "module")]
  pub module: String,
}
