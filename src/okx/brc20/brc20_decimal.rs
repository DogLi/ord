use super::fixed_point::{self, FixedPoint};

use std::cmp::{Ord, Ordering, PartialOrd};
use std::fmt::Display;
use std::ops::{Add, Div, Mul, Rem, Sub};
use std::str::FromStr;

use bigdecimal::num_bigint::BigInt;
use bigdecimal::BigDecimal;
use bigdecimal::{ParseBigDecimalError, Zero};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Brc20Decimal(BigDecimal);

impl Brc20Decimal {
  pub fn from_bigint(v: BigInt, scale: u8) -> Result<Self, ParseBigDecimalError> {
    if scale > FixedPoint::MAX_SCALE {
      return Err(ParseBigDecimalError::Other(format!(
        "scale {} exceeds the maximum allowed limit",
        scale
      )));
    }
    Ok(Self(BigDecimal::new(v, scale as i64)))
  }

  pub fn from_decimal(v: BigDecimal) -> Result<Self, ParseBigDecimalError> {
    let (v, scale) = v.into_bigint_and_scale();
    Self::from_bigint(v, scale as u8)
  }

  pub fn from_u128(v: u128, scale: u8) -> Result<Self, ParseBigDecimalError> {
    Self::from_bigint(BigInt::from(v), scale)
  }

  pub fn to_decimal(&self) -> &BigDecimal {
    &self.0
  }

  pub fn to_u128(&self) -> (u128, u8) {
    let (v, scale) = self.0.clone().into_bigint_and_scale();
    let s = v.to_biguint().unwrap().to_string();
    (s.parse::<u128>().unwrap(), scale as u8)
  }

  pub fn to_bigint(&self) -> (BigInt, u8) {
    let (v, scale) = self.0.clone().into_bigint_and_scale();
    (v, scale as u8)
  }

  pub fn sqrt(&self) -> Option<Self> {
    let v = self.0.clone().sqrt();
    if v.is_none() {
      return None;
    }
    Some(Self(v.unwrap()))
  }

  pub fn sign(&self) -> i8 {
    let sign = self.0.cmp(&BigDecimal::zero());
    if sign == Ordering::Greater {
      return 1;
    } else if sign == Ordering::Equal {
      return 0;
    } else {
      return -1;
    }
  }

  pub fn from_fix_point(fixed_point: FixedPoint) -> Self {
    Self(fixed_point.to_big_decimal())
  }

  pub fn get_precition(&self) -> u8 {
    self.0.clone().into_bigint_and_scale().1 as u8
  }

  fn check_scale_match(&self, other: &Self) -> bool {
    self.0.clone().into_bigint_and_scale().1 == other.0.clone().into_bigint_and_scale().1
  }
}

impl Serialize for Brc20Decimal {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    // 将 BigDecimal 转换为字符串进行序列化
    serializer.serialize_str(&self.0.to_string())
  }
}

impl<'de> Deserialize<'de> for Brc20Decimal {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s: String = Deserialize::deserialize(deserializer)?;
    // 从字符串重新构造 BigDecimal
    BigDecimal::from_str(&s)
      .map(Brc20Decimal)
      .map_err(|e| serde::de::Error::custom(format!("Invalid BigDecimal: {}", e)))
  }
}

impl FromStr for Brc20Decimal {
  type Err = ParseBigDecimalError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.is_empty() {
      return Err(ParseBigDecimalError::Empty);
    }
    let parts = s.split('.').collect::<Vec<&str>>();
    if parts.len() > 2 ||
            parts[0].is_empty() ||
            s.ends_with('.') || // "123." is invalid
            parts[0].chars().nth(0).unwrap() == '+' ||  // "+123.456" is invalid
            (parts.len() > 1 && parts[1].len() > FixedPoint::MAX_SCALE as usize)
    {
      return Err(ParseBigDecimalError::Other(
        "invalid brc20 decimal format".to_string(),
      ));
    }
    Ok(Self(BigDecimal::from_str(s.trim_end_matches('0'))?)) // remove trailing 0
  }
}

impl Ord for Brc20Decimal {
  fn cmp(&self, other: &Self) -> Ordering {
    self.0.cmp(&other.0)
  }
}

impl PartialOrd for Brc20Decimal {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Add for Brc20Decimal {
  type Output = Brc20Decimal;

  fn add(self, other: Self) -> Self::Output {
    if !self.check_scale_match(&other) {
      panic!("scale not match");
    }
    Self(self.0 + other.0)
  }
}

impl Sub for Brc20Decimal {
  type Output = Brc20Decimal;

  fn sub(self, other: Self) -> Self::Output {
    if !self.check_scale_match(&other) {
      panic!("scale not match");
    }
    Self(self.0 - other.0)
  }
}

impl Mul for Brc20Decimal {
  type Output = Brc20Decimal;

  fn mul(self, other: Self) -> Self::Output {
    Self(self.0 * other.0)
  }
}

impl Div for Brc20Decimal {
  type Output = Brc20Decimal;

  fn div(self, other: Self) -> Self::Output {
    Self(self.0 / other.0)
  }
}

impl Rem for Brc20Decimal {
  type Output = Brc20Decimal;

  fn rem(self, other: Self) -> Self::Output {
    Self(self.0 % other.0)
  }
}

impl Display for Brc20Decimal {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      self
        .0
        .to_string()
        .trim_end_matches('0')
        .trim_end_matches('.')
    )
  }
}

#[cfg(test)]
mod tests {
  use std::u128;

  use super::*;

  #[test]
  fn test_from_u128_zero() {
    let result = Brc20Decimal::from_u128(0, 0)
      .unwrap()
      .add(Brc20Decimal::from_u128(1, 0).unwrap());
    assert_eq!(result.to_string(), "1");
  }

  #[test]
  fn test_from_u128() {
    let cases: Vec<(u128, u8, &str)> = vec![
      (1000, 3, "1"),
      (123456, 2, "1234.56"),
      (1000, 18, "0.000000000000000001"),
    ];

    for (input, scale, expected) in cases {
      let result = Brc20Decimal::from_u128(input, scale);
      match result {
        Ok(brc20_decimal) => {
          assert_eq!(
            brc20_decimal.to_string(),
            expected,
            "input: {}, scale: {}",
            input,
            scale
          );
        }
        Err(_) => {
          panic!("unexpected error for input: {}, scale: {}", input, scale);
        }
      }
    }
  }

  #[test]
  fn test_from_str() {
    let test_cases = vec![
      // valid cases
      ("123456789.123456789", "123456789.123456789", false),
      ("123456789.123", "123456789.123", false),
      ("123456789", "123456789", false),
      ("-123456789.123456789", "-123456789.123456789", false),
      ("-123456789.123", "-123456789.123", false),
      ("-123456789", "-123456789", false),
      ("000001", "1", false),
      ("000001.1", "1.1", false),
      ("000001.100000000000000000", "1.1", false),
      // invalid cases
      ("", "", true),
      (" ", "", true),
      (".", "", true),
      (" 123.456", "", true),
      (".456", "", true),
      (".456 ", "", true),
      (" .456 ", "", true),
      (" 456", "", true),
      ("456 ", "", true),
      ("45 6", "", true),
      ("123. 456", "", true),
      ("123.-456", "", true),
      ("123.+456", "", true),
      ("+123.456", "", true),
      ("123.456.789", "", true),
      ("123456789.", "", true),
      ("123456789.12345678901234567891", "", true),
      ("0.1000000000000000000", "", true),
    ];

    for (input, expected, should_err) in test_cases {
      let result = Brc20Decimal::from_str(input);
      match result {
        Ok(brc20_decimal) => {
          if should_err {
            panic!("expected error for input: '{}'", input);
          } else {
            assert_eq!(brc20_decimal.to_string(), expected, "input: '{}'", input);
          }
        }
        Err(_) => {
          if !should_err {
            panic!("unexpected error for input: '{}'", input);
          }
        }
      }
    }
  }

  #[test]
  fn test_new_brc20_decimal() {
    let cases: Vec<(u128, u8, &str, bool)> = vec![
      (123456789, 8, "1.23456789", false),
      (123456789000000000000000000, 18, "123456789", false),
      // invalid cases
      (1234567892, 19, "", true),
    ];

    for (input, scale, expected, should_err) in cases {
      let result = Brc20Decimal::from_u128(input, scale);
      match result {
        Ok(brc20_decimal) => {
          if should_err {
            panic!("expected error for input: '{}'", input);
          } else {
            assert_eq!(brc20_decimal.to_string(), expected, "input: '{}'", input);
          }
        }
        Err(_) => {
          if !should_err {
            panic!("unexpected error for input: '{}'", input);
          }
        }
      }
    }
  }

  #[test]
  fn test_brc20_decimal_add() {
    let cases: Vec<(u128, u128, u8, &str)> = vec![(123412, 100, 2, "1235.12")];

    for (a, b, scale, expect) in cases {
      let a = Brc20Decimal::from_u128(a, scale).unwrap();
      let b = Brc20Decimal::from_u128(b, scale).unwrap();
      let c = a.clone() + b.clone();
      assert_eq!(
        c.to_string(),
        expect,
        "input: a={}, b={}",
        a.to_string(),
        b.to_string()
      );
    }
  }

  #[test]
  #[should_panic]
  fn test_brc20_decimal_add_scale_not_match() {
    let a = Brc20Decimal::from_u128(123412, 2).unwrap();
    let b = Brc20Decimal::from_u128(100, 0).unwrap();
    let _ = a.clone() + b.clone();
  }

  #[test]
  fn test_brc20_decimal_sub() {
    let cases: Vec<(u128, u128, u8, &str)> = vec![(123412, 100, 2, "1233.12")];

    for (a, b, scale, expect) in cases {
      let a = Brc20Decimal::from_u128(a, scale).unwrap();
      let b = Brc20Decimal::from_u128(b, scale).unwrap();
      let c = a.clone() - b.clone();
      assert_eq!(
        c.to_string(),
        expect,
        "input: a={}, b={}",
        a.to_string(),
        b.to_string()
      );
    }
  }

  #[test]
  #[should_panic]
  fn test_brc20_decimal_sub_scale_not_match() {
    let a = Brc20Decimal::from_u128(123412, 2).unwrap();
    let b = Brc20Decimal::from_u128(100, 0).unwrap();
    let _ = a.clone() - b.clone();
  }

  #[test]
  fn test_brc20_decimal_mul() {
    let cases: Vec<(u128, u128, u8, &str)> = vec![
      (123456712, 100, 2, "1234567.12"),
      (123456712, 1000, 2, "12345671.2"),
      (
        123456789000000000000000000,
        10000000000000000000,
        18,
        "1234567890",
      ),
      (
        u128::MAX,
        u128::MAX,
        0,
        "115792089237316195423570985008687907852589419931798687112530834793049593217025",
      ),
      (
        u128::MAX,
        u128::MAX,
        18,
        "115792089237316195423570985008687907852589.419931798687112530834793049593217025",
      ),
    ];

    for (a, b, scale, expected_str) in cases {
      let a = Brc20Decimal::from_u128(a, scale).unwrap();
      let b = Brc20Decimal::from_u128(b, scale).unwrap();
      let c = a.clone() * b.clone();
      assert_eq!(
        c.to_string(),
        expected_str,
        "input: a={}, b={}",
        a.to_string(),
        b.to_string()
      );
    }
  }

  #[test]
  fn test_brc20_decimal_mul_scale_not_match() {
    let a = Brc20Decimal::from_u128(123456712, 2).unwrap();
    let b = Brc20Decimal::from_u128(100, 0).unwrap();
    let c = a.clone() * b.clone();
    assert_eq!(
      c.to_string(),
      "123456712",
      "input: a={}, b={}",
      a.to_string(),
      b.to_string()
    );
  }

  #[test]
  fn test_brc20_decimal_div() {
    let cases: Vec<(u128, u128, u8, &str)> = vec![
      (123456712, 100, 2, "1234567.12"),
      (u128::MAX, u128::MAX, 0, "1"),
      (u128::MAX, u128::MAX, 18, "1"),
    ];

    for (a, b, scale, expected_str) in cases {
      let a = Brc20Decimal::from_u128(a, scale).unwrap();
      let b = Brc20Decimal::from_u128(b, scale).unwrap();
      let c = a.clone() / b.clone();
      assert_eq!(
        c.to_string(),
        expected_str,
        "input: a={}, b={}",
        a.to_string(),
        b.to_string()
      );
    }
  }

  #[test]
  fn test_brc20_decimal_sqrt() {
    let cases: Vec<(Brc20Decimal, String)> = vec![
      (Brc20Decimal::from_str("1.1").unwrap(), "1.1".to_string()),
      (
        Brc20Decimal::from_u128(u128::MAX, 0).unwrap(),
        u128::MAX.to_string(),
      ),
      (
        Brc20Decimal::from_u128(u128::MAX, 18).unwrap(),
        Brc20Decimal::from_u128(u128::MAX, 18).unwrap().to_string(),
      ),
    ];

    for (input, expected_str) in cases {
      let rv = (input.clone() * input.clone()).sqrt();
      if rv.is_none() {
        panic!("sqrt failed for input: {}", input.to_string());
      }
      assert_eq!(
        rv.unwrap().to_string(),
        expected_str,
        "input={}",
        input.to_string()
      );
    }
  }

  #[test]
  fn test_from_u128_with_scale() {
    let cases: Vec<(u128, u8, &str)> = vec![(123456789, 2, "1234567.89")];
    for (input, scale, expected) in cases {
      let result = Brc20Decimal::from_u128(input, scale);
      match result {
        Ok(brc20_decimal) => {
          assert_eq!(
            brc20_decimal.to_string(),
            expected,
            "input: {}, scale: {}",
            input,
            scale
          );
        }
        Err(_) => {
          panic!("unexpected error for input: {}, scale: {}", input, scale);
        }
      }
    }
  }
}
