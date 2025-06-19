use super::fixed_point::FixedPoint;

use std::cmp::{Ord, Ordering, PartialOrd};
use std::fmt::Display;
use std::ops::{Add, Div, Mul, Rem, Sub};
use std::str::FromStr;

use bigdecimal::num_bigint::{self, BigInt};
use bigdecimal::BigDecimal;
use bigdecimal::{ParseBigDecimalError, Zero};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::sync::LazyLock;

static PRECISION_FACTOR: LazyLock<[BigInt; 19]> = LazyLock::new(|| {
  [
    BigInt::from(10).pow(0),
    BigInt::from(10).pow(1),
    BigInt::from(10).pow(2),
    BigInt::from(10).pow(3),
    BigInt::from(10).pow(4),
    BigInt::from(10).pow(5),
    BigInt::from(10).pow(6),
    BigInt::from(10).pow(7),
    BigInt::from(10).pow(8),
    BigInt::from(10).pow(9),
    BigInt::from(10).pow(10),
    BigInt::from(10).pow(11),
    BigInt::from(10).pow(12),
    BigInt::from(10).pow(13),
    BigInt::from(10).pow(14),
    BigInt::from(10).pow(15),
    BigInt::from(10).pow(16),
    BigInt::from(10).pow(17),
    BigInt::from(10).pow(18),
  ]
});

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
    let (v, _) = self.0.clone().into_bigint_and_scale();
    let big_decimal = BigDecimal::from_bigint(v.sqrt(), 18);
    Some(Self(big_decimal))
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

  pub fn get_precision(&self) -> u8 {
    self.0.clone().into_bigint_and_scale().1 as u8
  }

  fn check_scale_match(&self, other: &Self) -> bool {
    self.0.clone().into_bigint_and_scale().1 == other.0.clone().into_bigint_and_scale().1
  }

  pub fn from_str_with_scale(s: &str, max_precision: u8) -> Result<Self, ParseBigDecimalError> {
    if s.is_empty() {
      return Err(ParseBigDecimalError::Empty);
    }

    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() > 2 {
      return Err(ParseBigDecimalError::Other(format!(
        "invalid decimal format: {}",
        s
      )));
    }

    let integer_part_str = parts[0];
    if integer_part_str.is_empty() || integer_part_str.starts_with('+') {
      return Err(ParseBigDecimalError::Other("empty integer".to_string()));
    }

    let integer_part = match BigInt::from_str(integer_part_str) {
      Ok(n) => n,
      Err(_) => {
        return Err(ParseBigDecimalError::Other(format!(
          "invalid integer format: {}",
          integer_part_str
        )))
      }
    };

    let mut decimal_part = BigInt::from(0);
    let mut curr_precision = 0;

    if parts.len() == 2 {
      let decimal_part_str = parts[1];
      if decimal_part_str.is_empty()
        || decimal_part_str.starts_with('+')
        || decimal_part_str.starts_with('-')
      {
        return Err(ParseBigDecimalError::Other("empty decimal".to_string()));
      }

      curr_precision = decimal_part_str.len();
      if curr_precision > max_precision.into() {
        return Err(ParseBigDecimalError::Other(format!(
          "decimal exceeds maximum precision: {}",
          s
        )));
      }

      let mut padded_decimal = decimal_part_str.to_string();
      let usize_max_precision: usize = max_precision.into();
      padded_decimal.push_str(&"0".repeat(usize_max_precision - curr_precision));

      decimal_part = match BigInt::from_str(&padded_decimal) {
        Ok(n) => n,
        Err(_) => {
          return Err(ParseBigDecimalError::Other(format!(
            "invalid decimal format: {}",
            decimal_part_str
          )))
        }
      };
    }

    //let scaled_integer = integer_part * BigInt::from(10).pow(max_precision as u32);
    let scaled_integer = integer_part.clone() * &PRECISION_FACTOR[max_precision as usize];
    let merged_value = if integer_part.sign() == num_bigint::Sign::Minus {
      scaled_integer - decimal_part
    } else {
      scaled_integer + decimal_part
    };

    let big_decimal = BigDecimal::new(merged_value, max_precision as i64);

    Ok(Brc20Decimal(big_decimal))
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
      panic!(
        "scale not match {:?} {:?}",
        self.get_precision(),
        other.get_precision()
      );
    }
    Self(self.0 + other.0)
  }
}

impl Sub for Brc20Decimal {
  type Output = Brc20Decimal;

  fn sub(self, other: Self) -> Self::Output {
    if !self.check_scale_match(&other) {
      panic!(
        "scale not match {:?} {:?}",
        self.get_precision(),
        other.get_precision()
      );
    }
    Self(self.0 - other.0)
  }
}

impl Mul for Brc20Decimal {
  type Output = Brc20Decimal;

  fn mul(self, other: Self) -> Self::Output {
    // let precition = self.get_precition();
    let (value0, scale0) = self.0.into_bigint_and_scale();
    let (value_other, _) = other.0.into_bigint_and_scale();
    let big_decimal = BigDecimal::from_bigint(value0 * value_other, scale0);
    Self(big_decimal)
  }
}

impl Div for Brc20Decimal {
  type Output = Brc20Decimal;

  fn div(self, other: Self) -> Self::Output {
    let (value0, scale0) = self.0.into_bigint_and_scale();
    let (value_other, _) = other.0.into_bigint_and_scale();
    let big_decimal = BigDecimal::from_bigint(value0 / value_other, scale0);
    Self(big_decimal)
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
    write!(f, "{}", self.0.to_string())
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

  #[test]
  fn test_from_str_with_scale() {
    let result = Brc20Decimal::from_str_with_scale("123", 8);
    println!("result: {:?}", result);
  }

  #[test]
  fn test_sqrt() {
    let decimal = Brc20Decimal::from_str_with_scale("1", 2).unwrap();
    println!(
      "decimal: {:?},{:?}",
      decimal.sqrt(),
      decimal.sqrt().unwrap().get_precision()
    );
    let d = Brc20Decimal::from_str_with_scale("1", 18).unwrap();
    let s = decimal.sqrt().unwrap() - d;
    println!("decimal: {:?}", s);

    let amt1 = Brc20Decimal::from_str_with_scale("10000", 8).unwrap();
    let amt2 = Brc20Decimal::from_str_with_scale("10000", 18).unwrap();

    println!(
      "amt1: {:?} {:?}",
      amt1.to_decimal().to_string(),
      amt1.clone().0.into_bigint_and_scale()
    );

    let x = amt1.mul(amt2);
    println!("x: {:?}", x.clone().0.into_bigint_and_scale());

    let q = x.sqrt().unwrap();
    println!("q: {:?}", q.clone().0.into_bigint_and_scale());

    let c = q.sub(Brc20Decimal::from_u128(1000, 18).unwrap());
    println!("c: {:?}", c.to_string());
    assert_eq!(
      c,
      Brc20Decimal::from_str_with_scale("0.099999999999999", 18).unwrap()
    );
  }

  #[test]
  fn test_to_u128() {
    let decimal = Brc20Decimal::from_str_with_scale("1234567890", 18).unwrap();
    println!("decimal: {:?}", decimal.to_decimal().to_string());
    let (value, scale) = decimal.to_u128();
    println!("value: {:?}, scale: {:?}", value, scale);
  }

  #[test]
  fn test_() {
    let d1 = Brc20Decimal::from_str_with_scale("1000000000000", 8).unwrap();
    let d2 = Brc20Decimal::from_str_with_scale("10000000000000000000000", 18).unwrap();
    println!("d1: {:?}", d1.clone().to_decimal().to_string());
    println!("d2: {:?}", d2.clone().to_decimal().to_string());

    let d1_mul_d2 = d1.clone().mul(d2.clone());
    println!(
      "d1_mul_d2: {:?}",
      d1_mul_d2.clone().to_decimal().to_string()
    );
    let d1_mul_d2_sqrt = d1_mul_d2.clone().sqrt().unwrap();
    println!(
      "d1_mul_d2 sqrt: {:?}",
      d1_mul_d2_sqrt.clone().to_decimal().to_string()
    );

    let d3 = Brc20Decimal::from_str_with_scale("1000", 18).unwrap();
    let d1_mul_d2_sqrt_sub_d3 = d1_mul_d2_sqrt.clone().sub(d3.clone());
    println!(
      "d1_mul_d2 sqrt sub d3: {:?}",
      d1_mul_d2_sqrt_sub_d3.clone().to_decimal().to_string()
    );

    let d4 = Brc20Decimal::from_str_with_scale("10000000000000000000000", 18).unwrap();
    let q = d1_mul_d2_sqrt_sub_d3.clone().div(d4.clone());
    println!("q: {:?}", q.clone().to_decimal().to_string());
  }

  #[test]
  fn test_mul_() {
    let d1 = Brc20Decimal::from_str_with_scale("1", 8).unwrap();
    let d2 = Brc20Decimal::from_str_with_scale("1", 18).unwrap();
    println!("d1: {:?}", d1.clone().to_string());
    println!("d2: {:?}", d2.clone().to_string());
    let d3 = d1.clone().mul(d2.clone());
    println!("d3: {:?}", d3.clone().to_string());
    let d4 = Brc20Decimal::from_str_with_scale("1", 18).unwrap();

    let (v, s) = d3.to_u128();
    println!("v: {:?}, s: {:?}", v, s);
    let (v2, s2) = d4.to_u128();
    println!("v2: {:?}, s2: {:?}", v2, s2);
  }

  #[test]
  fn test_fixed_point() {
    let amt = FixedPoint::new_from_str("77925.59776823", 8).unwrap();
    println!("amt: {:?}", amt);

    let amt2 = Brc20Decimal::from_fix_point(amt);
    println!("amt2: {:?}", amt2.to_string());
  }
}
