use super::*;
use serde_json::{json, Value};

pub(crate) mod commit;
mod create_module;
mod deploy;
mod mint;
mod transfer;
pub(crate) mod withdraw;

pub use self::{
  commit::Commit, create_module::CreateModule, deploy::Deploy, mint::Mint, transfer::Transfer,
  withdraw::Withdraw,
};

pub const PROTOCOL_LITERAL: &str = "brc-20";
pub const PROTOCOL_LITERAL_MODULE: &str = "brc20-module";
pub const PROTOCOL_LITERAL_SWAP: &str = "brc20-swap";

pub trait BRC20OperationExtractor {
  fn extract_brc20_operation(&self) -> Result<RawOperation, Error>;
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(tag = "op")]
pub enum RawOperation {
  #[serde(rename = "deploy")]
  Deploy(Deploy),
  #[serde(rename = "mint")]
  Mint(Mint),
  #[serde(rename = "transfer")]
  Transfer(Transfer),
  #[serde(rename = "create_module")] // unreachable!
  CreateModule(CreateModule),
  #[serde(rename = "withdraw")]
  Withdraw(Withdraw),
  #[serde(rename = "commit")]
  Commit(Commit),
}

impl BRC20OperationExtractor for Inscription {
  fn extract_brc20_operation(&self) -> Result<RawOperation, Error> {
    let content_body = self
      .body()
      .and_then(|body| std::str::from_utf8(body).ok())
      .ok_or(Error::InvalidJson)?;

    if content_body.len() < 40 {
      return Err(Error::NotBRC20Json);
    }

    let content_type = self.content_type().ok_or(Error::InvalidContentType)?;

    if content_type != "text/plain"
      && content_type != "text/plain;charset=utf-8"
      && content_type != "text/plain;charset=UTF-8"
      && content_type != "application/json"
      && !content_type.starts_with("text/plain;")
    {
      return Err(Error::UnSupportContentType);
    }

    deserialize_brc20_operation(content_body)
  }
}

fn deserialize_brc20_operation(s: &str) -> Result<RawOperation, Error> {
  let value: Value = serde_json::from_str(s).map_err(|_| Error::InvalidJson)?;
  if value.get("p") != Some(&json!(PROTOCOL_LITERAL))
    && value.get("p") != Some(&json!(PROTOCOL_LITERAL_MODULE))
    && value.get("p") != Some(&json!(PROTOCOL_LITERAL_SWAP))
  {
    return Err(Error::NotBRC20Json);
  }

  if value.get("p") == Some(&json!(PROTOCOL_LITERAL_MODULE))
    && value.get("op") == Some(&json!("deploy"))
  {
    // 解析 JSON 字符串到 CreateModule 结构体
    serde_json::from_value::<CreateModule>(value.clone())
      .map_err(|e| Error::ParseOperationJsonError(e.to_string()))
      .map(RawOperation::CreateModule)
  } else if value.get("p") == Some(&json!(PROTOCOL_LITERAL_MODULE))
    && value.get("op") == Some(&json!("withdraw"))
  {
    serde_json::from_value::<Withdraw>(value.clone())
      .map_err(|e| Error::ParseOperationJsonError(e.to_string()))
      .map(RawOperation::Withdraw)
  } else {
    serde_json::from_value(value).map_err(|e| Error::ParseOperationJsonError(e.to_string()))
  }
}

#[derive(PartialEq, Debug)]
pub enum Error {
  InvalidContentType,
  UnSupportContentType,
  InvalidJson,
  NotBRC20Json,
  ParseOperationJsonError(String),
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::InvalidContentType => write!(f, "Invalid content type"),
      Self::UnSupportContentType => write!(f, "Unsupported content type"),
      Self::InvalidJson => write!(f, "Invalid JSON string"),
      Self::NotBRC20Json => write!(f, "Not a valid BRC20 JSON"),
      Self::ParseOperationJsonError(err) => write!(f, "Failed to parse operation JSON: {}", err),
    }
  }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
  use crate::okx::brc20::operation::create_module::Init;

  use super::*;

  #[test]
  fn test_deploy_deserialize() {
    let max_supply = "21000000".to_string();
    let mint_limit = "1000".to_string();

    let json_str = format!(
      r##"{{
  "p": "brc-20",
  "op": "deploy",
  "tick": "ordi",
  "max": "{max_supply}",
  "lim": "{mint_limit}"
}}"##
    );

    assert_eq!(
      deserialize_brc20_operation(&json_str).unwrap(),
      RawOperation::Deploy(Deploy {
        tick: "ordi".to_string(),
        max_supply,
        mint_limit: Some(mint_limit),
        decimals: None,
        self_mint: None,
      })
    );
  }

  #[test]
  fn test_mint_deserialize() {
    let amount = "1000".to_string();

    let json_str = format!(
      r##"{{
  "p": "brc-20",
  "op": "mint",
  "tick": "ordi",
  "amt": "{amount}"
}}"##
    );

    assert_eq!(
      deserialize_brc20_operation(&json_str).unwrap(),
      RawOperation::Mint(Mint {
        tick: "ordi".to_string(),
        amount,
      })
    );
  }

  #[test]
  fn test_transfer_deserialize() {
    let amount = "100".to_string();

    let json_str = format!(
      r##"{{
  "p": "brc-20",
  "op": "transfer",
  "tick": "ordi",
  "amt": "{amount}"
}}"##
    );

    assert_eq!(
      deserialize_brc20_operation(&json_str).unwrap(),
      RawOperation::Transfer(Transfer {
        tick: "ordi".to_string(),
        amount,
      })
    );
  }

  #[test]
  fn test_json_duplicate_field() {
    let json_str = r#"{"p":"brc-20","op":"mint","tick":"smol","amt":"333","amt":"33"}"#;
    assert_eq!(
      deserialize_brc20_operation(json_str).unwrap(),
      RawOperation::Mint(Mint {
        tick: String::from("smol"),
        amount: String::from("33"),
      })
    )
  }

  #[test]
  fn test_missing_required_key() {
    assert_eq!(
      deserialize_brc20_operation(r#"{"p":"brc-20","op":"transfer","tick":"abcd"}"#).unwrap_err(),
      Error::ParseOperationJsonError("missing field `amt`".to_string())
    );
  }

  #[test]
  fn test_json_non_string() {
    let json_str = r#"{"p":"brc-20","op":"mint","tick":"smol","amt":33}"#;
    assert!(deserialize_brc20_operation(json_str).is_err())
  }

  #[test]
  fn test_deserialize_case_insensitive() {
    let max_supply = "21000000".to_string();
    let mint_limit = "1000".to_string();

    let json_str = format!(
      r##"{{
  "P": "brc-20",
  "Op": "deploy",
  "Tick": "ordi",
  "mAx": "{max_supply}",
  "Lim": "{mint_limit}"
}}"##
    );

    assert_eq!(
      deserialize_brc20_operation(&json_str),
      Err(Error::NotBRC20Json)
    );
  }

  #[test]
  fn test_duplicate_key() {
    let json_str = r#"{"p":"brc-20","op":"deploy","tick":"smol","max":"100","lim":"10","dec":"17","max":"200","lim":"20","max":"300"}"#;
    assert_eq!(
      deserialize_brc20_operation(json_str).unwrap(),
      RawOperation::Deploy(Deploy {
        tick: "smol".to_string(),
        max_supply: "300".to_string(),
        mint_limit: Some("20".to_string()),
        decimals: Some("17".to_string()),
        self_mint: None,
      })
    );

    let json_str = r#"{"p":"brc-20","op":"mint","tick":"smol","amt":"100","tick":"hhaa","amt":"200","tick":"actt"}"#;
    assert_eq!(
      deserialize_brc20_operation(json_str).unwrap(),
      RawOperation::Mint(Mint {
        tick: "actt".to_string(),
        amount: "200".to_string(),
      })
    );

    let json_str = r#"{"p":"brc-20","op":"transfer","tick":"smol","amt":"100","tick":"hhaa","amt":"200","tick":"actt"}"#;
    assert_eq!(
      deserialize_brc20_operation(json_str).unwrap(),
      RawOperation::Transfer(Transfer {
        tick: "actt".to_string(),
        amount: "200".to_string(),
      })
    );
  }

  #[test]
  fn test_brc20_swap() {
    let json_str = r#"{"p":"brc20-module","op":"deploy","name":"swap","source":"34761af5d6aa0ab7f14589e6f0a905b505c40ba542c9a27d407c9d052499ffddi0","init":{"swap_fee_rate":"0.003","gas_tick":"bSATS_","gas_to":"bc1qam880mjcygnkjny5km39vut89vsnq7yun4nr73","fee_to":"bc1qaejzncrr87pr7azn9fadwa79cp42acsxeygert","sequencer":"bc1qtaf86fqf9hv7r76927fjpxc0mpvedgyp7zjneu"}}"#;
    assert_eq!(
      deserialize_brc20_operation(json_str).unwrap(),
      RawOperation::CreateModule(CreateModule {
        name: "swap".to_string(),
        source: "34761af5d6aa0ab7f14589e6f0a905b505c40ba542c9a27d407c9d052499ffddi0".to_string(),
        init: Init {
          swap_fee_rate: "0.003".to_string(),
          gas_tick: "bSATS_".to_string(),
          gas_to: "bc1qam880mjcygnkjny5km39vut89vsnq7yun4nr73".to_string(),
          fee_to: "bc1qaejzncrr87pr7azn9fadwa79cp42acsxeygert".to_string(),
          sequencer: "bc1qtaf86fqf9hv7r76927fjpxc0mpvedgyp7zjneu".to_string(),
        },
      })
    );
  }

  #[test]
  fn test_brc20_swap_withdraw() {
    let json_str =
      r#"{"p":"brc20-module","op":"withdraw","tick":"ordi","module":"module","amt":"10"}"#;
    println!("{:?}", deserialize_brc20_operation(json_str).unwrap());
  }

  #[test]
  fn test_brc20_swap_commit() {
    let json_str = r#"{"p":"brc20-swap","op":"commit","module":"module","parent":"xxxxi0","gas_price":"100","data":[{"func":"send","params":["bc1qaejzncrr87pr7azn9fadwa79cp42acsxeygert","ordi","10"],"addr":"bc1q...", "ts":12345,"sig":"sig"}]}"#;
    println!("{:?}", deserialize_brc20_operation(json_str).unwrap());
  }
}
