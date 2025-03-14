use super::*;
use bitcoin::{
  address::{Address, NetworkUnchecked},
  Script, ScriptHash,
};

pub(crate) type UtxoAddressRef = UtxoAddressInner;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum UtxoAddressInner {
  Address(Address<NetworkUnchecked>),
  ScriptHash {
    op_return: bool,
    script_hash: ScriptHash,
    script_buf: ScriptBuf,
  },
}

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UtxoAddress(UtxoAddressInner);

impl UtxoAddress {
  pub fn from_script(script: &Script, chain: &Chain) -> Self {
    Self(
      chain
        .address_from_script(script)
        .map(|address| UtxoAddressInner::Address(address.as_unchecked().clone()))
        .unwrap_or(UtxoAddressInner::ScriptHash {
          script_hash: script.script_hash(),
          op_return: script.is_op_return(),
          script_buf: script.to_owned(),
        }),
    )
  }

  pub fn to_script(&self, chain: &Chain) -> ScriptBuf {
    match &self.0 {
      UtxoAddressInner::Address(address) => {
        chain.to_script_pubkey(&address.clone().require_network(chain.network()).unwrap())
      }
      UtxoAddressInner::ScriptHash { script_buf, .. } => script_buf.clone(),
    }
  }

  pub fn from_str(address: &str, network: Network) -> Result<Self> {
    Ok(
      Address::from_str(address)?
        .require_network(network)
        .map(|address| Self(UtxoAddressInner::Address(address.as_unchecked().clone())))?,
    )
  }

  pub fn from_address(address: Address) -> Self {
    let inner = UtxoAddressInner::Address(address.as_unchecked().clone());
    Self(inner)
  }

  pub fn op_return(&self) -> bool {
    match &self.0 {
      UtxoAddressInner::Address(_) => false,
      UtxoAddressInner::ScriptHash { op_return, .. } => *op_return,
    }
  }

  pub fn get_script_by_str(address: &str, chain: &Chain) -> Result<ScriptBuf> {
    let address = Self::from_str(address, chain.network())?;
    Ok(address.to_script(chain))
  }

  pub(crate) fn as_ref(&self) -> &UtxoAddressRef {
    &self.0
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use bitcoin::{Address, Script};
  use std::str::FromStr;

  #[test]
  fn test_from_script_with_valid_address() {
    let address = Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap();
    assert_eq!(
      UtxoAddress::from_script(
        address.clone().assume_checked().script_pubkey().as_script(),
        &Chain::Mainnet
      ),
      UtxoAddress(UtxoAddressInner::Address(address))
    );
  }

  #[test]
  fn test_from_script_with_non_op_return_script() {
    let hex_script = hex::decode(
      "0014017fed86bba5f31f955f8b316c7fb9bd45cb6cbc00000000000000000000000000000000000000",
    )
    .expect("Failed to decode hex script");
    let script = Script::from_bytes(hex_script.as_slice());

    assert_eq!(
      UtxoAddress::from_script(script, &Chain::Mainnet),
      UtxoAddress(UtxoAddressInner::ScriptHash {
        script_hash: ScriptHash::from_str("df65c8a338dce7900824e7bd18c336656ca19e57")
          .expect("Failed to parse script hash"),
        op_return: false,
        script_buf: ScriptBuf::from_bytes(hex_script)
      })
    );
  }

  #[test]
  fn test_from_script_with_op_return_script() {
    let hex_script =
      hex::decode("6a0b68656c6c6f20776f726c64").expect("Failed to decode hex script");
    let script = Script::from_bytes(hex_script.as_slice());

    assert_eq!(
      UtxoAddress::from_script(script, &Chain::Mainnet),
      UtxoAddress(UtxoAddressInner::ScriptHash {
        script_hash: ScriptHash::from_str("70c382a01444e96a1fd2eeb9041bdef603e0c410")
          .expect("Failed to parse script hash"),
        op_return: true,
        script_buf: ScriptBuf::from_bytes(hex_script)
      })
    );
  }

  #[test]
  fn test_serialize_deserialize_with_address() {
    let descriptor = UtxoAddress(UtxoAddressInner::Address(
      Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap(),
    ));

    let serialized = bincode::serialize(&descriptor).unwrap();
    assert_eq!(
      serialized,
      vec![
        0, 0, 0, 0, 42, 0, 0, 0, 0, 0, 0, 0, 98, 99, 49, 113, 104, 118, 100, 54, 115, 117, 118,
        113, 122, 106, 99, 117, 57, 112, 120, 106, 104, 114, 119, 104, 116, 114, 108, 106, 56, 53,
        110, 121, 51, 110, 50, 109, 113, 113, 108, 53, 119, 52
      ]
    );
    let deserialized: UtxoAddress = bincode::deserialize(&serialized).unwrap();

    assert_eq!(deserialized, descriptor);
  }

  #[test]
  fn test_serialize_deserialize_with_non_op_return_script() {
    let hex_script = hex::decode(
      "0014017fed86bba5f31f955f8b316c7fb9bd45cb6cbc00000000000000000000000000000000000000",
    )
    .expect("Failed to decode hex script");
    let descriptor = UtxoAddress(UtxoAddressInner::ScriptHash {
      script_hash: ScriptHash::from_str("df65c8a338dce7900824e7bd18c336656ca19e57").unwrap(),
      op_return: false,
      script_buf: ScriptBuf::from_bytes(hex_script),
    });

    let serialized = bincode::serialize(&descriptor).unwrap();
    assert_eq!(
      serialized,
      vec![
        1, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 223, 101, 200, 163, 56, 220, 231, 144, 8, 36, 231,
        189, 24, 195, 54, 101, 108, 161, 158, 87, 41, 0, 0, 0, 0, 0, 0, 0, 0, 20, 1, 127, 237, 134,
        187, 165, 243, 31, 149, 95, 139, 49, 108, 127, 185, 189, 69, 203, 108, 188, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
      ]
    );
    let deserialized: UtxoAddress = bincode::deserialize(&serialized).unwrap();

    assert_eq!(deserialized, descriptor);
  }

  #[test]
  fn test_serialize_deserialize_with_op_return_script() {
    let hex_script =
      hex::decode("6a0b68656c6c6f20776f726c64").expect("Failed to decode hex script");
    let descriptor = UtxoAddress(UtxoAddressInner::ScriptHash {
      script_hash: ScriptHash::from_str("70c382a01444e96a1fd2eeb9041bdef603e0c410").unwrap(),
      op_return: true,
      script_buf: ScriptBuf::from_bytes(hex_script),
    });

    let serialized = bincode::serialize(&descriptor).unwrap();
    assert_eq!(
      serialized,
      vec![
        1, 0, 0, 0, 1, 20, 0, 0, 0, 0, 0, 0, 0, 112, 195, 130, 160, 20, 68, 233, 106, 31, 210, 238,
        185, 4, 27, 222, 246, 3, 224, 196, 16, 13, 0, 0, 0, 0, 0, 0, 0, 106, 11, 104, 101, 108,
        108, 111, 32, 119, 111, 114, 108, 100
      ]
    );
    let deserialized: UtxoAddress = bincode::deserialize(&serialized).unwrap();

    assert_eq!(deserialized, descriptor);
  }

  #[test]
  fn test_get_script_by_str() {
    let address = UtxoAddress::from_str(
      "bc1qam880mjcygnkjny5km39vut89vsnq7yun4nr73",
      Chain::Mainnet.into(),
    )
    .unwrap();
    let script = UtxoAddress::get_script_by_str(
      "bc1qam880mjcygnkjny5km39vut89vsnq7yun4nr73",
      &Chain::Mainnet,
    )
    .unwrap();
    println!(
      "script: {:?} \n {:?}",
      script.to_string(),
      script.to_hex_string()
    );

    let addree_from_script = UtxoAddress::from_script(
      &Script::from_bytes(script.clone().as_bytes()),
      &Chain::Mainnet,
    );

    assert_eq!(address, addree_from_script);

    let sb = ScriptBuf::from_hex("0014eece77ee582227694c94b6e25671672b2130789c").unwrap();
    let addree_from_str_script = UtxoAddress::from_script(sb.as_script(), &Chain::Mainnet);

    assert_eq!(address, addree_from_str_script);
  }
}
