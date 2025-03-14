use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CreateModule {
  #[serde(rename = "name")]
  pub name: String,
  #[serde(rename = "source")]
  pub source: String,

  #[serde(rename = "init")]
  pub init: Init,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Init {
  #[serde(rename = "swap_fee_rate")]
  pub swap_fee_rate: String,
  #[serde(rename = "gas_tick")]
  pub gas_tick: String,
  #[serde(rename = "gas_to")]
  pub gas_to: String,
  #[serde(rename = "fee_to")]
  pub fee_to: String,
  #[serde(rename = "sequencer")]
  pub sequencer: String,
}
