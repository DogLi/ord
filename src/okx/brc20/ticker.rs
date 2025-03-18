use super::*;

#[derive(Debug, PartialEq, Clone, PartialOrd, Ord, Eq, SerializeDisplay, DeserializeFromStr)]
pub struct BRC20Ticker(Box<[u8]>);

impl BRC20Ticker {
  pub const MIN_SIZE: usize = 6;
  pub const MAX_SIZE: usize = 12;

  pub const TICKER_B63: [u8; 256] = [
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 255, 255, 255,
    255, 255, 255, 255, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
    29, 30, 31, 32, 33, 34, 35, 255, 255, 255, 255, 36, 255, 37, 38, 39, 40, 41, 42, 43, 44, 45,
    46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
  ];

  pub fn len(&self) -> usize {
    self.0.len()
  }

  pub fn to_lowercase(&self) -> BRC20LowerCaseTicker {
    let str = self.to_string().to_lowercase();
    BRC20LowerCaseTicker(str.as_bytes().to_vec().into_boxed_slice())
  }
}

impl Display for BRC20Ticker {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", std::str::from_utf8(&self.0).unwrap())
  }
}

impl FromStr for BRC20Ticker {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let bytes = s.as_bytes();
    let length = bytes.len();

    // BRC20Ticker names on the Fractal Bitcoin mainnet will be limited to 6 - 12 bytes.
    if !(Self::MIN_SIZE..=Self::MAX_SIZE).contains(&length) {
      return Err(Error::Range);
    }

    // Fractal Bitcoin limits the ticker characters:
    for c in bytes {
      if BRC20Ticker::TICKER_B63[*c as usize] > 63 {
        return Err(Error::InvalidChar);
      }
    }

    Ok(Self(bytes.into()))
  }
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Ord, Eq, SerializeDisplay, Deserialize)]
pub struct BRC20LowerCaseTicker(Box<[u8]>);

impl BRC20LowerCaseTicker {
  pub fn len(&self) -> usize {
    self.0.len()
  }

  pub fn to_box(&self) -> Box<[u8]> {
    self.0.clone()
  }
}

impl Display for BRC20LowerCaseTicker {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", std::str::from_utf8(&self.0).unwrap())
  }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub enum Error {
  Range,
  InvalidChar,
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Range => write!(f, "ticker name out of range"),
      Self::InvalidChar => write!(f, "ticker characters invalid"),
    }
  }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_ticker_from_str_valid_bytes() {
    assert!(BRC20Ticker::from_str("BTC").is_err()); // length is less than MIN_SIZE
    assert!(BRC20Ticker::from_str("BITOIN").is_err()); // length is greater than MAX_SIZE

    assert_eq!(BRC20Ticker::from_str("ORDI").unwrap().to_string(), "ORDI"); // length is 4 bytes
    assert_eq!(BRC20Ticker::from_str("USDTS").unwrap().to_string(), "USDTS"); // length is 5 bytes
  }

  #[test]
  fn test_ticker_from_str_invalid_bytes() {
    assert_eq!(BRC20Ticker::from_str(""), Err(Error::Range));
    assert_eq!(BRC20Ticker::from_str("XAİİ"), Err(Error::Range));
  }

  #[test]
  fn test_ticker_from_str_valid() {
    // 4 bytes
    assert!(BRC20Ticker::from_str("XAİ").is_ok());
    assert!(BRC20Ticker::from_str("X。").is_ok());
    assert!(BRC20Ticker::from_str("aBc1").is_ok());
    assert!(BRC20Ticker::from_str("ατ").is_ok());
    assert!(BRC20Ticker::from_str("∑i").is_ok());
    assert!(BRC20Ticker::from_str("⊢i").is_ok());
    assert!(BRC20Ticker::from_str("≯a").is_ok());

    // 5 bytes
    assert!(BRC20Ticker::from_str("∑ii").is_ok());
    assert!(BRC20Ticker::from_str("⊢ii").is_ok());
    assert!(BRC20Ticker::from_str("a≯a").is_ok());
  }

  #[test]
  fn test_ticker_from_str_invalid() {
    assert_eq!(BRC20Ticker::from_str(""), Err(Error::Range));
    assert_eq!(BRC20Ticker::from_str("BTC"), Err(Error::Range));
    assert_eq!(BRC20Ticker::from_str("BITCOI"), Err(Error::Range));
    assert_eq!(BRC20Ticker::from_str("XAİİ"), Err(Error::Range));
  }

  #[test]
  fn test_ticker_to_lowercase() {
    assert_eq!(
      BRC20Ticker::from_str("aBc1")
        .unwrap()
        .to_lowercase()
        .to_string(),
      "abc1"
    );
    assert_eq!(
      BRC20Ticker::from_str("XAİ")
        .unwrap()
        .to_lowercase()
        .to_string(),
      "xai\u{307}"
    );
    assert_eq!(
      BRC20Ticker::from_str("ατ")
        .unwrap()
        .to_lowercase()
        .to_string(),
      "ατ"
    );
    assert_eq!(
      BRC20Ticker::from_str("∑H")
        .unwrap()
        .to_lowercase()
        .to_string(),
      "∑h"
    );
    assert_eq!(
      BRC20Ticker::from_str("⊢I")
        .unwrap()
        .to_lowercase()
        .to_string(),
      "⊢i"
    );
    assert_eq!(
      BRC20Ticker::from_str("≯A")
        .unwrap()
        .to_lowercase()
        .to_string(),
      "≯a"
    );
  }

  #[test]
  fn test_lowercase_ticker_display() {
    let lower = BRC20LowerCaseTicker(b"ordi".to_vec().into_boxed_slice());
    assert_eq!(format!("{}", lower), "ordi");

    let lower = BRC20LowerCaseTicker(b"sats".to_vec().into_boxed_slice());
    assert_eq!(format!("{}", lower), "sats");
  }

  #[test]
  fn test_ticker_display() {
    let ticker = BRC20Ticker::from_str("ORDI").unwrap();
    assert_eq!(format!("{}", ticker), "ORDI");

    let ticker = BRC20Ticker::from_str("SATS").unwrap();
    assert_eq!(format!("{}", ticker), "SATS");
  }

  #[test]
  fn test_ticker_len() {
    let ticker = BRC20Ticker::from_str("BTCD").unwrap();
    assert_eq!(ticker.len(), 4);

    let ticker = BRC20Ticker::from_str("USDTU").unwrap();
    assert_eq!(ticker.len(), 5);
  }

  #[test]
  fn test_ticker_equality() {
    let ticker1 = BRC20Ticker::from_str("BTCD").unwrap();
    let ticker2 = BRC20Ticker::from_str("BTCD").unwrap();
    let ticker3 = BRC20Ticker::from_str("USDT").unwrap();

    assert_eq!(ticker1, ticker2);
    assert_ne!(ticker1, ticker3);
  }

  #[test]
  fn test_ticker_ordering() {
    let ticker1 = BRC20Ticker::from_str("BTCD").unwrap();
    let ticker2 = BRC20Ticker::from_str("ETHI").unwrap();
    let ticker3 = BRC20Ticker::from_str("USDTT").unwrap();

    assert!(ticker1 < ticker2);
    assert!(ticker2 < ticker3);
  }

  #[test]
  fn test_error_display() {
    let ticker1 = BRC20Ticker::from_str("BTCDDD");
    assert_eq!(
      format!("{}", ticker1.err().unwrap()),
      "ticker name out of range"
    );
  }

  #[test]
  fn test_lowercase_ticker_from_bytes() {
    let lower = BRC20LowerCaseTicker(b"ordi".to_vec().into_boxed_slice());
    assert_eq!(lower.to_string(), "ordi");
    let lower = BRC20LowerCaseTicker("xai\u{307}".as_bytes().to_vec().into_boxed_slice());
    assert_eq!(lower.to_string(), "xai\u{307}");
  }

  #[test]
  fn test_tick_serialize() {
    let obj = BRC20Ticker::from_str("Ab1;").unwrap();
    let serialized = bincode::serialize(&obj).unwrap();
    assert_eq!(serialized, vec![4, 0, 0, 0, 0, 0, 0, 0, 65, 98, 49, 59]);

    let lower = obj.to_lowercase();
    let serialized = bincode::serialize(&lower).unwrap();
    assert_eq!(serialized, vec![4, 0, 0, 0, 0, 0, 0, 0, 97, 98, 49, 59]);

    let obj = BRC20Ticker::from_str("XXAİ").unwrap();
    let serialized = bincode::serialize(&obj).unwrap();
    assert_eq!(
      serialized,
      vec![5, 0, 0, 0, 0, 0, 0, 0, 88, 88, 65, 196, 176]
    );

    let lower = obj.to_lowercase();
    let serialized = bincode::serialize(&lower).unwrap();
    assert_eq!(
      serialized,
      vec![6, 0, 0, 0, 0, 0, 0, 0, 120, 120, 97, 105, 204, 135]
    );
  }

  #[test]
  fn test_tick_deserialize() {
    let obj = BRC20Ticker::from_str("Ab1;").unwrap();
    let deserialized =
      bincode::deserialize::<BRC20Ticker>(&[4, 0, 0, 0, 0, 0, 0, 0, 65, 98, 49, 59]).unwrap();

    assert_eq!(deserialized, obj);

    let lower = obj.to_lowercase();
    let deserialized =
      bincode::deserialize::<BRC20LowerCaseTicker>(&[4, 0, 0, 0, 0, 0, 0, 0, 97, 98, 49, 59])
        .unwrap();

    assert_eq!(deserialized, lower);

    let obj = BRC20Ticker::from_str("XXAİ").unwrap();
    let deserialized =
      bincode::deserialize::<BRC20Ticker>(&[5, 0, 0, 0, 0, 0, 0, 0, 88, 88, 65, 196, 176]).unwrap();

    assert_eq!(deserialized, obj);

    let lower = obj.to_lowercase();
    let deserialized = bincode::deserialize::<BRC20LowerCaseTicker>(&[
      6, 0, 0, 0, 0, 0, 0, 0, 120, 120, 97, 105, 204, 135,
    ])
    .unwrap();

    assert_eq!(deserialized, lower);

    // deserialize with error
    assert_eq!(
      bincode::deserialize::<BRC20Ticker>(&[6, 0, 0, 0, 0, 0, 0, 0, 65, 98, 49, 59, 47, 49])
        .unwrap_err()
        .to_string(),
      Error::Range.to_string()
    );
  }

  #[test]
  fn test_tick_serialize_json() {
    let obj = BRC20Ticker::from_str("Ab1;").unwrap();
    let serialized = serde_json::to_string(&obj).unwrap();
    assert_eq!(serialized, "\"Ab1;\"");

    let lower = obj.to_lowercase();
    let serialized = serde_json::to_string(&lower).unwrap();
    assert_eq!(serialized, "\"ab1;\"");

    let obj = BRC20Ticker::from_str("XXAİ").unwrap();
    let serialized = serde_json::to_string(&obj).unwrap();
    assert_eq!(serialized, "\"XXAİ\"");

    let lower = obj.to_lowercase();
    let serialized = serde_json::to_string(&lower).unwrap();
    assert_eq!(serialized, "\"xxai\u{307}\"");
  }

  #[test]
  fn test_ticker_char_validation() {
    assert!(BRC20Ticker::from_str("hello___").is_ok());
    assert!(BRC20Ticker::from_str("hello_world").is_ok());
    assert!(BRC20Ticker::from_str("1029asdf_").is_ok());
    assert!(BRC20Ticker::from_str("&sdfsdf").is_err());
    assert!(BRC20Ticker::from_str("1029a(sdf_").is_err());
    assert!(BRC20Ticker::from_str("!llo_world_").is_err());
    assert!(BRC20Ticker::from_str("GLIZZY_").is_ok());
  }
}
