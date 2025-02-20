use crate::{InscriptionId, SatPoint};
use bitcoin::{Address, BlockHash, Network, Txid};
use reqwest::blocking::Client;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use url::Url;

/// the request Client for unisat
/// https://open-api.unisat.io/swagger.html
pub struct UnisatClient {
  token: String,
  client: Client,
}

impl UnisatClient {
  const SERVER_ADDR: &'static str = "https://open-api-fractal.unisat.io";
  pub fn new(token: &str) -> Self {
    let client = Client::new();
    Self {
      token: token.to_string(),
      client,
    }
  }

  // get withdraw history from [start, end)
  pub fn get_withdraw_history(
    &self,
    block_hash: BlockHash,
  ) -> anyhow::Result<UnisatResponse<DataWithdrawHistory>> {
    let uri = format!(
      "/v1/indexer/brc20-module/withdraw-history-by-block-id/{}",
      block_hash
    );
    let url = Url::parse(&format!("{}{uri}", Self::SERVER_ADDR)).unwrap();
    log::info!("query url: {}", url);

    let token = format!("Bearer {}", self.token);
    let builder = self
      .client
      .request(Method::GET, url)
      .timeout(std::time::Duration::from_secs(10))
      .header(reqwest::header::AUTHORIZATION, &token)
      .header(reqwest::header::CONTENT_TYPE, "application/json");

    match builder.send() {
      Ok(response) => {
        let payload = response.text()?;
        match serde_json::from_str(&payload) {
          Ok(result) => return Ok(result),
          Err(e) => {
            log::error!("get invalid data for request withdraw history: {payload}");
            anyhow::bail!(e)
          }
        };
      }
      Err(e) => {
        log::error!("request withdraw history failed: {e:?}");
        anyhow::bail!(e)
      }
    };
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawHistory {
  #[serde(rename = "type")]
  pub tx_type: String,
  pub valid: bool,
  pub txid: Txid,
  pub idx: u64,
  pub vout: u64,
  pub offset: usize,
  pub inscription_number: i32,
  pub inscription_id: InscriptionId,
  pub content_type: String,
  pub content_body: String,
  pub old_sat_point: SatPoint,
  pub new_sat_point: SatPoint,
  pub from: String,
  pub to: String,
  pub satoshi: u64,
  pub data: TickInfo,
  pub height: u32,
  pub txidx: u64,
  pub blockhash: BlockHash,
  pub blocktime: i64,
}

impl WithdrawHistory {
  pub fn from_address(&self) -> anyhow::Result<Address> {
    let address = Address::from_str(&self.from)?.require_network(Network::Bitcoin)?;
    Ok(address)
  }
  pub fn to_address(&self) -> anyhow::Result<Address> {
    let address = Address::from_str(&self.to)?.require_network(Network::Bitcoin)?;
    Ok(address)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TickInfo {
  pub tick: String,
  #[serde(rename = "amount")]
  pub readable_amount: String, //
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataWithdrawHistory {
  pub height: u32,
  pub total: usize,
  pub cursor: usize,
  pub detail: Vec<WithdrawHistory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnisatResponse<T> {
  pub code: i32,
  pub msg: String,
  pub data: Option<T>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_unisat_response() {
    let s = r#"
        {
    "code": 0,
    "msg": "ok",
    "data": {
        "height": 88644,
        "total": 45,
        "cursor": 0,
        "detail": [
            {
                "type": "withdraw",
                "valid": true,
                "txid": "bd331d228d959479da5a100f26917cdd11dc99a69c65f77228092d359696381e",
                "idx": 0,
                "vout": 0,
                "offset": 0,
                "inscriptionNumber": 115441310,
                "inscriptionId": "7ac3fd332d0ce02a3773d50ed532caea154fc7ac9afca08d7bb5a69cc18486c0i0",
                "contentType": "",
                "contentBody": "{\"p\":\"brc20-module\",\"op\":\"withdraw\",\"tick\":\"GLIZZY\",\"amt\":\"8.99999999\",\"module\":\"fd5bd482bed1b62d0702e2f19a1e3bdd4fb755fa5c9bed5d8d0f219a3219ee95i0\"}",
                "oldSatPoint": "7ac3fd332d0ce02a3773d50ed532caea154fc7ac9afca08d7bb5a69cc18486c0:0:0",
                "newSatPoint": "bd331d228d959479da5a100f26917cdd11dc99a69c65f77228092d359696381e:0:0",
                "from": "bc1q76qc3rxsy5v764n24q6un3n9yzfker8y9dmrxy",
                "to": "bc1q76qc3rxsy5v764n24q6un3n9yzfker8y9dmrxy",
                "satoshi": 330,
                "data": {
                    "tick": "GLIZZY",
                    "amount": "8.99999999"
                },
                "height": 51759,
                "txidx": 5036,
                "blockhash": "000000000000000000118c7b6e61e630584a07529aed5e21a02f4e0653d5970b",
                "blocktime": 1727345136
            },
            {
                "type": "withdraw",
                "valid": true,
                "txid": "ca082a883cac91ea89bad8e2e6348737f3e8010399370535587bab994964bced",
                "idx": 0,
                "vout": 0,
                "offset": 0,
                "inscriptionNumber": 115555176,
                "inscriptionId": "d466d8f8250f88bc18763a7ea431ee09efb95b641df90ecee79e950fdc1c2f27i0",
                "contentType": "",
                "contentBody": "{\"p\":\"brc20-module\",\"op\":\"withdraw\",\"tick\":\"bSATS_\",\"amt\":\"8.363799614177412983\",\"module\":\"fd5bd482bed1b62d0702e2f19a1e3bdd4fb755fa5c9bed5d8d0f219a3219ee95i0\"}",
                "oldSatPoint": "d466d8f8250f88bc18763a7ea431ee09efb95b641df90ecee79e950fdc1c2f27:0:0",
                "newSatPoint": "ca082a883cac91ea89bad8e2e6348737f3e8010399370535587bab994964bced:0:0",
                "from": "bc1q76qc3rxsy5v764n24q6un3n9yzfker8y9dmrxy",
                "to": "bc1q76qc3rxsy5v764n24q6un3n9yzfker8y9dmrxy",
                "satoshi": 330,
                "data": {
                    "tick": "bSATS_",
                    "amount": "8.363799614177412983"
                },
                "height": 51780,
                "txidx": 1947,
                "blockhash": "000000000000000001c9f17cf9c0f2dbf7fe526acd6130f6b47e51f38b66e754",
                "blocktime": 1727345946
            },
            {
                "type": "withdraw",
                "valid": true,
                "txid": "800fb37ad761e6526614efc5a820d7d41e4d1aa12222151e1a0e0569f2f3f82d",
                "idx": 0,
                "vout": 0,
                "offset": 0,
                "inscriptionNumber": 115613049,
                "inscriptionId": "cea9972aeaae9fb75154ab046161b137a2a83d03baff869c98ebea916c507540i0",
                "contentType": "",
                "contentBody": "{\"p\":\"brc20-module\",\"op\":\"withdraw\",\"tick\":\"GLIZZY\",\"amt\":\"9.993\",\"module\":\"fd5bd482bed1b62d0702e2f19a1e3bdd4fb755fa5c9bed5d8d0f219a3219ee95i0\"}",
                "oldSatPoint": "cea9972aeaae9fb75154ab046161b137a2a83d03baff869c98ebea916c507540:0:0",
                "newSatPoint": "800fb37ad761e6526614efc5a820d7d41e4d1aa12222151e1a0e0569f2f3f82d:0:0",
                "from": "bc1ph8m57xc93q4hsntj5085g84y3g87fry28ann5r98r7pq482vzsjqujjxhh",
                "to": "bc1ph8m57xc93q4hsntj5085g84y3g87fry28ann5r98r7pq482vzsjqujjxhh",
                "satoshi": 330,
                "data": {
                    "tick": "GLIZZY",
                    "amount": "9.993"
                },
                "height": 51790,
                "txidx": 1118,
                "blockhash": "000000000000000004652b8911ad2b4eed379c2d70bc1825973bfbe7aef7269a",
                "blocktime": 1727346210
            },
            {
                "type": "withdraw",
                "valid": true,
                "txid": "96640b8773ecb72837ff221a959282beb3683fc1471dac8a5e7e5f8f022c3f3b",
                "idx": 0,
                "vout": 0,
                "offset": 0,
                "inscriptionNumber": 125983267,
                "inscriptionId": "0eb8ffc78faa2d29bb460675d56cf38fcdc4252a52e210ed20dcdff7808f7ddbi0",
                "contentType": "",
                "contentBody": "{\"p\":\"brc20-module\",\"op\":\"withdraw\",\"tick\":\"GLIZZY\",\"amt\":\"1\",\"module\":\"fd5bd482bed1b62d0702e2f19a1e3bdd4fb755fa5c9bed5d8d0f219a3219ee95i0\"}",
                "oldSatPoint": "0eb8ffc78faa2d29bb460675d56cf38fcdc4252a52e210ed20dcdff7808f7ddb:0:0",
                "newSatPoint": "96640b8773ecb72837ff221a959282beb3683fc1471dac8a5e7e5f8f022c3f3b:0:0",
                "from": "bc1q76qc3rxsy5v764n24q6un3n9yzfker8y9dmrxy",
                "to": "bc1q76qc3rxsy5v764n24q6un3n9yzfker8y9dmrxy",
                "satoshi": 330,
                "data": {
                    "tick": "GLIZZY",
                    "amount": "1"
                },
                "height": 54877,
                "txidx": 2,
                "blockhash": "0000000000000000055cd3897054c74deed7dfae71d32e0a22e8b8c1f34689da",
                "blocktime": 1727437327
            },
            {
                "type": "withdraw",
                "valid": true,
                "txid": "14b50c36fb2607a31ea764ce54042f55a4b42e9d2a3cf8aeda32eac13e01e20a",
                "idx": 0,
                "vout": 0,
                "offset": 0,
                "inscriptionNumber": 126053002,
                "inscriptionId": "e48008c39201801bdebca6daa8e41e6c05362680a197dff711864cca2ef045a8i0",
                "contentType": "",
                "contentBody": "{\"p\":\"brc20-module\",\"op\":\"withdraw\",\"tick\":\"GLIZZY\",\"amt\":\"1\",\"module\":\"fd5bd482bed1b62d0702e2f19a1e3bdd4fb755fa5c9bed5d8d0f219a3219ee95i0\"}",
                "oldSatPoint": "e48008c39201801bdebca6daa8e41e6c05362680a197dff711864cca2ef045a8:0:0",
                "newSatPoint": "14b50c36fb2607a31ea764ce54042f55a4b42e9d2a3cf8aeda32eac13e01e20a:0:0",
                "from": "bc1q76qc3rxsy5v764n24q6un3n9yzfker8y9dmrxy",
                "to": "bc1q76qc3rxsy5v764n24q6un3n9yzfker8y9dmrxy",
                "satoshi": 330,
                "data": {
                    "tick": "GLIZZY",
                    "amount": "1"
                },
                "height": 54890,
                "txidx": 2,
                "blockhash": "00000000000000000257409133ae7212109ba4ac105ab6e78b3229899826d473",
                "blocktime": 1727437682
            },
            {
                "type": "withdraw",
                "valid": true,
                "txid": "529f51d64184cd5faf334c36d82bf887dac83c8d6f8fa7baf8e0624590f5b46e",
                "idx": 0,
                "vout": 0,
                "offset": 0,
                "inscriptionNumber": 126053003,
                "inscriptionId": "5e24228ed46b3cb077ab9d76fab141212452e643186215e2566d81190d8ad2d1i0",
                "contentType": "",
                "contentBody": "{\"p\":\"brc20-module\",\"op\":\"withdraw\",\"tick\":\"GLIZZY\",\"amt\":\"1\",\"module\":\"fd5bd482bed1b62d0702e2f19a1e3bdd4fb755fa5c9bed5d8d0f219a3219ee95i0\"}",
                "oldSatPoint": "5e24228ed46b3cb077ab9d76fab141212452e643186215e2566d81190d8ad2d1:0:0",
                "newSatPoint": "529f51d64184cd5faf334c36d82bf887dac83c8d6f8fa7baf8e0624590f5b46e:0:0",
                "from": "bc1q76qc3rxsy5v764n24q6un3n9yzfker8y9dmrxy",
                "to": "bc1q76qc3rxsy5v764n24q6un3n9yzfker8y9dmrxy",
                "satoshi": 330,
                "data": {
                    "tick": "GLIZZY",
                    "amount": "1"
                },
                "height": 54890,
                "txidx": 4,
                "blockhash": "00000000000000000257409133ae7212109ba4ac105ab6e78b3229899826d473",
                "blocktime": 1727437682
            },
            {
                "type": "withdraw",
                "valid": true,
                "txid": "7b0212c8ce1dc551841f3277a03d4bb838b2062254e7351e6fdf2d4d37cd5f5a",
                "idx": 0,
                "vout": 0,
                "offset": 0,
                "inscriptionNumber": 126101197,
                "inscriptionId": "5d36323d48c9dfaa947f0f2f14fd6be99c8c7389f7045db2a49e278cf3f72191i0",
                "contentType": "",
                "contentBody": "{\"p\":\"brc20-module\",\"op\":\"withdraw\",\"tick\":\"GLIZZY\",\"amt\":\"1\",\"module\":\"fd5bd482bed1b62d0702e2f19a1e3bdd4fb755fa5c9bed5d8d0f219a3219ee95i0\"}",
                "oldSatPoint": "5d36323d48c9dfaa947f0f2f14fd6be99c8c7389f7045db2a49e278cf3f72191:0:0",
                "newSatPoint": "7b0212c8ce1dc551841f3277a03d4bb838b2062254e7351e6fdf2d4d37cd5f5a:0:0",
                "from": "bc1q76qc3rxsy5v764n24q6un3n9yzfker8y9dmrxy",
                "to": "bc1q76qc3rxsy5v764n24q6un3n9yzfker8y9dmrxy",
                "satoshi": 330,
                "data": {
                    "tick": "GLIZZY",
                    "amount": "1"
                },
                "height": 54899,
                "txidx": 2,
                "blockhash": "0000000000000000036a760ed4f41ab3cad89c9a7df64b2df5ee159f52997226",
                "blocktime": 1727438022
            },
            {
                "type": "withdraw",
                "valid": true,
                "txid": "10329c875c7c2a60493a75a8ec81234c844cb0150f7cbbe7fedc936e00f5bc7d",
                "idx": 0,
                "vout": 0,
                "offset": 0,
                "inscriptionNumber": 126145808,
                "inscriptionId": "0126a0830f219e0286e8f2fd8825201c9dee7bdc05840326589a0677873e358fi0",
                "contentType": "",
                "contentBody": "{\"p\":\"brc20-module\",\"op\":\"withdraw\",\"tick\":\"bSATS_\",\"amt\":\"1\",\"module\":\"fd5bd482bed1b62d0702e2f19a1e3bdd4fb755fa5c9bed5d8d0f219a3219ee95i0\"}",
                "oldSatPoint": "0126a0830f219e0286e8f2fd8825201c9dee7bdc05840326589a0677873e358f:0:0",
                "newSatPoint": "10329c875c7c2a60493a75a8ec81234c844cb0150f7cbbe7fedc936e00f5bc7d:0:0",
                "from": "bc1q76qc3rxsy5v764n24q6un3n9yzfker8y9dmrxy",
                "to": "bc1q76qc3rxsy5v764n24q6un3n9yzfker8y9dmrxy",
                "satoshi": 330,
                "data": {
                    "tick": "bSATS_",
                    "amount": "1"
                },
                "height": 54909,
                "txidx": 3,
                "blockhash": "87e080d7ca071e736f35db28b7deb3da4e23336c33750781405bef6341054556",
                "blocktime": 1727438365
            },
            {
                "type": "withdraw",
                "valid": true,
                "txid": "fa36aa9ab1b5f904204b2f4135eea99f6d2446d5b2834a61e02dd900b4eba064",
                "idx": 0,
                "vout": 0,
                "offset": 0,
                "inscriptionNumber": 126187983,
                "inscriptionId": "e8ce7cb6b27c70509fd6e2d394605240ccc1445aeb60d09d6cd1efc8dec6a103i0",
                "contentType": "",
                "contentBody": "{\"p\":\"brc20-module\",\"op\":\"withdraw\",\"tick\":\"GLIZZY\",\"amt\":\"11\",\"module\":\"fd5bd482bed1b62d0702e2f19a1e3bdd4fb755fa5c9bed5d8d0f219a3219ee95i0\"}",
                "oldSatPoint": "e8ce7cb6b27c70509fd6e2d394605240ccc1445aeb60d09d6cd1efc8dec6a103:0:0",
                "newSatPoint": "fa36aa9ab1b5f904204b2f4135eea99f6d2446d5b2834a61e02dd900b4eba064:0:0",
                "from": "bc1ph8m57xc93q4hsntj5085g84y3g87fry28ann5r98r7pq482vzsjqujjxhh",
                "to": "bc1ph8m57xc93q4hsntj5085g84y3g87fry28ann5r98r7pq482vzsjqujjxhh",
                "satoshi": 330,
                "data": {
                    "tick": "GLIZZY",
                    "amount": "11"
                },
                "height": 54916,
                "txidx": 4,
                "blockhash": "02ebf63da58b663edadbced243d781f463ffdc8736f69b8a8426b29cd5f34634",
                "blocktime": 1727438529
            },
            {
                "type": "withdraw",
                "valid": true,
                "txid": "eff81e690d87a658b0fd27f006cc5a291b3502fd9bb4ec9c7610c9d20b99ed8e",
                "idx": 0,
                "vout": 0,
                "offset": 0,
                "inscriptionNumber": 126255892,
                "inscriptionId": "240bb35490c650fb9e0f2877d0bf1c855e3df1eba366b7355cd422286af84722i0",
                "contentType": "",
                "contentBody": "{\"p\":\"brc20-module\",\"op\":\"withdraw\",\"tick\":\"GLIZZY\",\"amt\":\"7\",\"module\":\"fd5bd482bed1b62d0702e2f19a1e3bdd4fb755fa5c9bed5d8d0f219a3219ee95i0\"}",
                "oldSatPoint": "240bb35490c650fb9e0f2877d0bf1c855e3df1eba366b7355cd422286af84722:0:0",
                "newSatPoint": "eff81e690d87a658b0fd27f006cc5a291b3502fd9bb4ec9c7610c9d20b99ed8e:0:0",
                "from": "bc1ph8m57xc93q4hsntj5085g84y3g87fry28ann5r98r7pq482vzsjqujjxhh",
                "to": "bc1ph8m57xc93q4hsntj5085g84y3g87fry28ann5r98r7pq482vzsjqujjxhh",
                "satoshi": 330,
                "data": {
                    "tick": "GLIZZY",
                    "amount": "7"
                },
                "height": 54928,
                "txidx": 4453,
                "blockhash": "3204bc27b828fe20d681691017664e109ada37bb87a21d9ff1dfa7461a10b086",
                "blocktime": 1727438917
            },
            {
                "type": "withdraw",
                "valid": true,
                "txid": "aec07fb36ebf51e2658fa3ce8f2111a0428f231880cf5689f3307daa2dd1d6a9",
                "idx": 0,
                "vout": 0,
                "offset": 0,
                "inscriptionNumber": 126281951,
                "inscriptionId": "3b4dcb2cf75b8ff0a14c2fe176855a275675437017490a58064499a7747bc3c9i0",
                "contentType": "",
                "contentBody": "{\"p\":\"brc20-module\",\"op\":\"withdraw\",\"tick\":\"bSATS_\",\"amt\":\"1\",\"module\":\"fd5bd482bed1b62d0702e2f19a1e3bdd4fb755fa5c9bed5d8d0f219a3219ee95i0\"}",
                "oldSatPoint": "3b4dcb2cf75b8ff0a14c2fe176855a275675437017490a58064499a7747bc3c9:0:0",
                "newSatPoint": "aec07fb36ebf51e2658fa3ce8f2111a0428f231880cf5689f3307daa2dd1d6a9:0:0",
                "from": "bc1q76qc3rxsy5v764n24q6un3n9yzfker8y9dmrxy",
                "to": "bc1q76qc3rxsy5v764n24q6un3n9yzfker8y9dmrxy",
                "satoshi": 330,
                "data": {
                    "tick": "bSATS_",
                    "amount": "1"
                },
                "height": 54934,
                "txidx": 2006,
                "blockhash": "3fe7e83a5512d9384dcc5c532543d9032e3d2107f5715e2f84cd2a94dd94d45e",
                "blocktime": 1727439210
            },
            {
                "type": "withdraw",
                "valid": true,
                "txid": "916f0f25353ee88e87f55ee1af0c01966f8590b62128e9eeb68eb05e9c913269",
                "idx": 0,
                "vout": 0,
                "offset": 0,
                "inscriptionNumber": 126339654,
                "inscriptionId": "839abc5ebd13779298039e3fb47ca6cfe4418d59af7433e08766fa2a45742c0fi0",
                "contentType": "",
                "contentBody": "{\"p\":\"brc20-module\",\"op\":\"withdraw\",\"tick\":\"GLIZZY\",\"amt\":\"7.7777777\",\"module\":\"fd5bd482bed1b62d0702e2f19a1e3bdd4fb755fa5c9bed5d8d0f219a3219ee95i0\"}",
                "oldSatPoint": "839abc5ebd13779298039e3fb47ca6cfe4418d59af7433e08766fa2a45742c0f:0:0",
                "newSatPoint": "916f0f25353ee88e87f55ee1af0c01966f8590b62128e9eeb68eb05e9c913269:0:0",
                "from": "bc1ph8m57xc93q4hsntj5085g84y3g87fry28ann5r98r7pq482vzsjqujjxhh",
                "to": "bc1ph8m57xc93q4hsntj5085g84y3g87fry28ann5r98r7pq482vzsjqujjxhh",
                "satoshi": 330,
                "data": {
                    "tick": "GLIZZY",
                    "amount": "7.7777777"
                },
                "height": 54944,
                "txidx": 3255,
                "blockhash": "000000000000000000f00f43cb964a248c42463cb3f788b8a96b34f23ec4e8d6",
                "blocktime": 1727439373
            },
            {
                "type": "withdraw",
                "valid": true,
                "txid": "bdfd4f94a41803c49535596853fca45f5dd8295b924e785de9de973942c55ed6",
                "idx": 0,
                "vout": 0,
                "offset": 0,
                "inscriptionNumber": 126339655,
                "inscriptionId": "863e64f9ae88b56c30c91e3a819faf9faa82811eb8aa4e5b3d34aa98c102a88di0",
                "contentType": "",
                "contentBody": "{\"p\":\"brc20-module\",\"op\":\"withdraw\",\"tick\":\"GLIZZY\",\"amt\":\"8.88888\",\"module\":\"fd5bd482bed1b62d0702e2f19a1e3bdd4fb755fa5c9bed5d8d0f219a3219ee95i0\"}",
                "oldSatPoint": "863e64f9ae88b56c30c91e3a819faf9faa82811eb8aa4e5b3d34aa98c102a88d:0:0",
                "newSatPoint": "bdfd4f94a41803c49535596853fca45f5dd8295b924e785de9de973942c55ed6:0:0",
                "from": "bc1ph8m57xc93q4hsntj5085g84y3g87fry28ann5r98r7pq482vzsjqujjxhh",
                "to": "bc1ph8m57xc93q4hsntj5085g84y3g87fry28ann5r98r7pq482vzsjqujjxhh",
                "satoshi": 330,
                "data": {
                    "tick": "GLIZZY",
                    "amount": "8.88888"
                },
                "height": 54944,
                "txidx": 3257,
                "blockhash": "000000000000000000f00f43cb964a248c42463cb3f788b8a96b34f23ec4e8d6",
                "blocktime": 1727439373
            },
            {
                "type": "withdraw",
                "valid": true,
                "txid": "a568cbe912c098743e1d0f32cba69ec5ad3ad4c125390ff9fec1a8c3a85826f4",
                "idx": 0,
                "vout": 0,
                "offset": 0,
                "inscriptionNumber": 126402170,
                "inscriptionId": "8e8900b0983ed1ed15beb3e6f8907298cde66c9bf52031f5a0b8968c56ce85d9i0",
                "contentType": "",
                "contentBody": "{\"p\":\"brc20-module\",\"op\":\"withdraw\",\"tick\":\"GLIZZY\",\"amt\":\"4.4444\",\"module\":\"fd5bd482bed1b62d0702e2f19a1e3bdd4fb755fa5c9bed5d8d0f219a3219ee95i0\"}",
                "oldSatPoint": "8e8900b0983ed1ed15beb3e6f8907298cde66c9bf52031f5a0b8968c56ce85d9:0:0",
                "newSatPoint": "a568cbe912c098743e1d0f32cba69ec5ad3ad4c125390ff9fec1a8c3a85826f4:0:0",
                "from": "bc1ph8m57xc93q4hsntj5085g84y3g87fry28ann5r98r7pq482vzsjqujjxhh",
                "to": "bc1ph8m57xc93q4hsntj5085g84y3g87fry28ann5r98r7pq482vzsjqujjxhh",
                "satoshi": 330,
                "data": {
                    "tick": "GLIZZY",
                    "amount": "4.4444"
                },
                "height": 54957,
                "txidx": 4323,
                "blockhash": "f926312c8b53d16af21835a1873f1c40cca96322c6cc73f9d0f6fda578aa1123",
                "blocktime": 1727439732
            },
            {
                "type": "withdraw",
                "valid": true,
                "txid": "36285de409a467f47f8e080d3765e052b74c8d6fbaca465368fed4b92b824ebb",
                "idx": 0,
                "vout": 0,
                "offset": 0,
                "inscriptionNumber": 126422712,
                "inscriptionId": "eabc5373bed89ab54c45298cd9f212bbe2575e108e72c85609654ba1c4d8a516i0",
                "contentType": "",
                "contentBody": "{\"p\":\"brc20-module\",\"op\":\"withdraw\",\"tick\":\"bSATS_\",\"amt\":\"1.22222\",\"module\":\"fd5bd482bed1b62d0702e2f19a1e3bdd4fb755fa5c9bed5d8d0f219a3219ee95i0\"}",
                "oldSatPoint": "eabc5373bed89ab54c45298cd9f212bbe2575e108e72c85609654ba1c4d8a516:0:0",
                "newSatPoint": "36285de409a467f47f8e080d3765e052b74c8d6fbaca465368fed4b92b824ebb:0:0",
                "from": "bc1ph8m57xc93q4hsntj5085g84y3g87fry28ann5r98r7pq482vzsjqujjxhh",
                "to": "bc1ph8m57xc93q4hsntj5085g84y3g87fry28ann5r98r7pq482vzsjqujjxhh",
                "satoshi": 330,
                "data": {
                    "tick": "bSATS_",
                    "amount": "1.22222"
                },
                "height": 54963,
                "txidx": 5315,
                "blockhash": "000000000000000005f93e7684b2463c13fb3dae8cb042505b47671c0c64bc81",
                "blocktime": 1727440105
            },
            {
                "type": "withdraw",
                "valid": true,
                "txid": "9ca2a6dc413845d860f913e810f84fa1154a34dde44a0989124c0cdc32823cf6",
                "idx": 0,
                "vout": 0,
                "offset": 0,
                "inscriptionNumber": 126422935,
                "inscriptionId": "5573884f55aea3bb069e52b3940aa3f23348893ab487a7e7e9fb8f23e67d72b5i0",
                "contentType": "",
                "contentBody": "{\"p\":\"brc20-module\",\"op\":\"withdraw\",\"tick\":\"GLIZZY\",\"amt\":\"1\",\"module\":\"fd5bd482bed1b62d0702e2f19a1e3bdd4fb755fa5c9bed5d8d0f219a3219ee95i0\"}",
                "oldSatPoint": "5573884f55aea3bb069e52b3940aa3f23348893ab487a7e7e9fb8f23e67d72b5:0:0",
                "newSatPoint": "9ca2a6dc413845d860f913e810f84fa1154a34dde44a0989124c0cdc32823cf6:0:0",
                "from": "bc1q76qc3rxsy5v764n24q6un3n9yzfker8y9dmrxy",
                "to": "bc1q76qc3rxsy5v764n24q6un3n9yzfker8y9dmrxy",
                "satoshi": 330,
                "data": {
                    "tick": "GLIZZY",
                    "amount": "1"
                },
                "height": 54964,
                "txidx": 73,
                "blockhash": "56afd734a4ad4bf41c07aa52113c6e10ae89cff0afac58b56b4835bb138aa698",
                "blocktime": 1727440114
            }
        ]
    }
}"#;
    let resp: Result<UnisatResponse<DataWithdrawHistory>, _> = serde_json::from_str(s);
    assert!(resp.is_ok());
    let data = resp.unwrap().data.unwrap();
    for withdraw in data.detail {
      assert!(withdraw.from_address().is_ok());
      assert!(withdraw.to_address().is_ok());
    }
  }

  #[test]
  fn test_unisat_client() {
    let _ = env_logger::builder().is_test(true).try_init();
    let token = std::env::var("UNISAT_API_TOKEN").expect("Expected UNISAT_API_TOKEN to be env var. See https://docs.unisat.io/dev/unisat-developer-center#getting-an-api-key");
    let client = UnisatClient::new(&token);
    let s = "\"0000000000000000029f8dc939488afd4a795f03714c3242a10d7b8181ab4c7a\"";
    let block_hash: BlockHash = serde_json::from_str(s).unwrap();
    let resp = client.get_withdraw_history(block_hash).unwrap();
    assert!(resp.data.is_some());
    println!("{:?}", resp.data.unwrap());
  }

  #[test]
  fn test_inscription_id() {
    let s = r#"
    {
                "type": "withdraw",
                "valid": true,
                "txid": "e415892fa827e4bcae6f95c98bd48c9f30906c5671618e5bca55f294d9539d54",
                "idx": 0,
                "vout": 0,
                "offset": 0,
                "inscriptionNumber": 284005008,
                "inscriptionId": "957765644213122bca1c453342eb860592979a7975395af33ce67605324474e5i0",
                "contentType": "",
                "contentBody": "{\"p\":\"brc20-module\",\"op\":\"withdraw\",\"tick\":\"sFB___000\",\"amt\":\"1\",\"module\":\"fd5bd482bed1b62d0702e2f19a1e3bdd4fb755fa5c9bed5d8d0f219a3219ee95i0\"}",
                "oldSatPoint": "957765644213122bca1c453342eb860592979a7975395af33ce67605324474e5:0:0",
                "newSatPoint": "e415892fa827e4bcae6f95c98bd48c9f30906c5671618e5bca55f294d9539d54:0:0",
                "from": "bc1qpn7fw7ftxw35363fgs6wf8kzxdc8c203a2gqw4",
                "to": "bc1qpn7fw7ftxw35363fgs6wf8kzxdc8c203a2gqw4",
                "satoshi": 330,
                "data": {
                    "tick": "sFB___000",
                    "amount": "1"
                },
                "height": 103801,
                "txidx": 2,
                "blockhash": "0000000000000000029f8dc939488afd4a795f03714c3242a10d7b8181ab4c7a",
                "blocktime": 1728914425
            }"#;
    let history: WithdrawHistory = serde_json::from_str(s).unwrap();
    println!("block hash:{}", history.blockhash);
    let inscription_id = history.inscription_id;
    let s1 = serde_json::to_string(&inscription_id).unwrap();
    println!("s1:{}", s1);
    println!("history id:{inscription_id:?}");
    let s = "\"957765644213122bca1c453342eb860592979a7975395af33ce67605324474e5i0\"".to_string();
    let id: InscriptionId = serde_json::from_str(&s).unwrap();
    println!("id: {id:?}");
    assert_eq!(inscription_id, id);
  }
}
