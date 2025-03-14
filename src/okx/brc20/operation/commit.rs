use crate::impl_bincode_dynamic_entry;
use crate::okx::entry::DynamicEntry;
use serde::{Deserialize, Serialize};

// SwapFunctionData
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SwapFunctionData {
  #[serde(rename = "addr", skip_serializing_if = "Option::is_none")]
  pub address: Option<String>,

  #[serde(rename = "func", skip_serializing_if = "Option::is_none")]
  pub function: Option<String>,

  #[serde(rename = "params", skip_serializing_if = "Option::is_none")]
  pub params: Option<Vec<String>>,

  #[serde(rename = "ts", skip_serializing_if = "Option::is_none")]
  pub timestamp: Option<u32>,

  #[serde(rename = "sig", skip_serializing_if = "Option::is_none")]
  pub signature: Option<String>,
  // #[serde(skip_serializing)]
  // pub id: String, // 不参与 JSON 序列化

  // #[serde(skip_serializing)]
  // pub pk_script: String, // 不参与 JSON 序列化
}

// Commit
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Commit {
  #[serde(rename = "module", skip_serializing_if = "Option::is_none")]
  pub module: Option<String>,

  #[serde(rename = "parent", skip_serializing_if = "Option::is_none")]
  pub parent: Option<String>,

  #[serde(rename = "gas_price", skip_serializing_if = "Option::is_none")]
  pub gas_price: Option<String>,

  #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
  pub data: Option<Vec<SwapFunctionData>>,
}

pub type CommitValue = [u8];
impl_bincode_dynamic_entry!(Commit, CommitValue);
