use super::{brc20::BRC20Ticker, *};
use byteorder::{ByteOrder, LittleEndian};

pub fn get_module_from_script(script: &ScriptBuf) -> Result<String, bool> {
  let script = script.as_bytes(); // 将 ScriptBuf 转换为字节切片
  let n = script.len();
  if n < 34 || n > 38 {
    return Err(false);
  }
  if script[0] != 0x6a {
    return Err(false);
  }
  if (script[1] as usize) + 2 != n {
    return Err(false);
  }

  // Remove trailing 0
  if n > 34 && script[n - 1] == 0 {
    return Err(false);
  }

  let idx = match script[1] {
    0..=32 => 0,
    33 => script[34] as u32,
    34 => LittleEndian::read_u16(&script[34..36]) as u32,
    35 => (script[34] as u32) | ((script[35] as u32) << 8) | ((script[36] as u32) << 16),
    36 => LittleEndian::read_u32(&script[34..38]),
    _ => return Err(false),
  };

  let hash = hash_string(&script[2..34]);
  let module = format!("{}i{}", hash, idx);
  Ok(module)
}

pub fn hash_string(data: &[u8]) -> String {
  let length = 32;
  let mut reverse_data = [0u8; 32];

  // 反转字节数组
  for i in 0..length {
    reverse_data[i] = data[length - i - 1];
  }

  // 将反转后的字节数组编码为十六进制字符串
  hex::encode(reverse_data)
}

pub fn get_valid_unique_lower_ticker(ticker: &str) -> Result<String, Error> {
  if ticker.len() < BRC20Ticker::MIN_SIZE {
    return Err(anyhow!("ticker is too short"));
  }
  if ticker.len() > BRC20Ticker::MAX_SIZE {
    return Err(anyhow!("ticker is too long"));
  }

  for c in ticker.chars() {
    if BRC20Ticker::TICKER_B63[c as usize] > 63 {
      return Err(anyhow!("ticker invalid"));
    }
  }

  let ticker = ticker.to_lowercase();
  Ok(ticker)
}

pub fn decode_tokens_from_swap_pair(tick_pair: &str) -> Result<(String, String), Error> {
  if let Some(slash_idx) = tick_pair.find('/') {
    if tick_pair.len() < BRC20Ticker::MIN_SIZE * 2 + 1
      || tick_pair.len() > BRC20Ticker::MAX_SIZE * 2 + 1
      || slash_idx < BRC20Ticker::MIN_SIZE
      || slash_idx > BRC20Ticker::MAX_SIZE
      || tick_pair.len() - (slash_idx + 1) < BRC20Ticker::MIN_SIZE
    {
      return Err(anyhow!("tickPair invalid"));
    }

    let token0 = tick_pair[..slash_idx].to_string();
    let token1 = tick_pair[slash_idx + 1..].to_string();

    Ok((token0, token1))
  } else {
    Err(anyhow!("tickPair invalid, no slash found"))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_module_from_script() {
    let binding =
      hex::decode("6a2095ee19329a210f8d5ded9b5cfa55b74fdd3b1e9af1e202072db6d1be82d45bfd").unwrap();
    let script = Script::from_bytes(binding.as_ref());
    let receive = UtxoAddress::from_script(&script, &Chain::Mainnet);

    let binding = receive.to_script(&Chain::Mainnet);
    let new_script = binding.as_script();
    assert_eq!(script, new_script);
    println!("script_buf: {:?}", new_script.to_bytes());
    let module_id = get_module_from_script(&receive.to_script(&Chain::Mainnet)).unwrap();
    assert_eq!(
      module_id,
      "fd5bd482bed1b62d0702e2f19a1e3bdd4fb755fa5c9bed5d8d0f219a3219ee95i0",
    );
  }
}
