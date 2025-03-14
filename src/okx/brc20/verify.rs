use super::*;

#[derive(PartialEq, Debug)]
pub enum Error {
  InvalidBrc20Ticker,
  InvalidBrc20Amount,
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::InvalidBrc20Ticker => write!(f, "invalid brc20 ticker"),
      Self::InvalidBrc20Amount => write!(f, "invalid brc20 amount"),
    }
  }
}

pub fn check_ticker_verify(
  context: &mut TableContext,
  ticker: &BRC20Ticker,
  amt_str: &String,
) -> Result<brc20_decimal::Brc20Decimal, Error> {
  let result = context.load_brc20_ticker_info(ticker).unwrap();
  if let Some(ticker_info) = result {
    if amt_str == "" {
      log::debug!(
        "brc20swap check_ticker_verify amt is empty, ticker: {:?}, amt: {:?}",
        ticker,
        amt_str
      );
      return Ok(brc20_decimal::Brc20Decimal::from_u128(0, ticker_info.decimals).unwrap());
    }

    // FixedPoint can not be negative
    let amt = match FixedPoint::new_from_str(&amt_str, ticker_info.decimals) {
      Ok(result) => brc20_decimal::Brc20Decimal::from_fix_point(result),
      Err(e) => {
        log::debug!(
          "brc20swap error check_ticker_verify amt invalid, ticker: {:?}, amt: {:?}, error: {:?}",
          ticker,
          amt_str,
          e
        );
        return Err(Error::InvalidBrc20Amount);
      }
    };

    if amt.cmp(&brc20_decimal::Brc20Decimal::from_fix_point(
      FixedPoint::new(ticker_info.total_supply, ticker_info.decimals).unwrap(),
    )) == std::cmp::Ordering::Greater
    {
      log::debug!(
        "brc20swap error check_ticker_verify amt exceed total supply, ticker: {:?}, amt: {:?}, total_supply: {:?}, decimals: {:?}",
        ticker,
        amt,
        ticker_info.total_supply,
        ticker_info.decimals
      );
      return Err(Error::InvalidBrc20Amount);
    }

    return Ok(amt);
  } else {
    log::debug!(
      "brc20swap error check_ticker_verify ticker not found, ticker: {:?}, amt: {:?}",
      ticker,
      amt_str
    );
    return Err(Error::InvalidBrc20Ticker);
  }
}
