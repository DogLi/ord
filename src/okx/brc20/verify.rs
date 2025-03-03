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
) -> Result<FixedPoint, Error> {
  let result = context.load_brc20_ticker_info(ticker).unwrap();
  if let Some(ticker_info) = result {
    if amt_str == "" {
      return Ok(FixedPoint::new_unchecked(0, 0));
    }

    // FixedPoint can not be negative
    let amt = match FixedPoint::new_from_str(&amt_str, ticker_info.decimals) {
      Ok(result) => result,
      Err(_) => return Err(Error::InvalidBrc20Amount),
    };

    if amt.cmp(&FixedPoint::new(ticker_info.max_mint_limit, ticker_info.decimals).unwrap())
      == std::cmp::Ordering::Greater
    {
      return Err(Error::InvalidBrc20Amount);
    }

    return Ok(amt);
  } else {
    return Err(Error::InvalidBrc20Ticker);
  }
}
