use std::collections::HashMap;
use {super::*, axum::Json};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiInscriptionDigest {
  pub id: InscriptionId,
  pub number: i64,
  pub location: SatPoint,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiOutPointResult {
  pub result: Option<ApiOutpointInscriptions>,
  pub latest_blockhash: BlockHash,
  pub latest_height: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiOutpointInscriptions {
  pub txid: Txid,
  pub script_pub_key: String,
  pub owner: ApiUtxoAddress,
  pub value: u64,
  pub inscription_digest: Vec<ApiInscriptionDigest>,
}

// /ord/outpoint/:outpoint/info
/// Retrieve the outpoint information with the specified outpoint.

pub(crate) async fn ord_outpoint(
  Extension(settings): Extension<Arc<Settings>>,
  Extension(index): Extension<Arc<Index>>,
  Path(outpoint): Path<OutPoint>,
) -> ApiResult<ApiOutPointResult> {
  log::debug!("rpc: get ord_outpoint: {outpoint}");
  task::block_in_place(|| {
    let rtx = index.begin_read()?;
    let Some((height, blockhash)) = Index::latest_block(&rtx)? else {
      return Err(OrdApiError::DataBaseNotReady.into());
    };

    let (inscriptions_with_satpoints, value, script_pubkey) =
      Index::get_inscriptions_on_output_with_satpoints_and_script_pubkey(outpoint, &rtx, &index)?;

    // If there are no inscriptions on the output, return None and parsed block states.
    if inscriptions_with_satpoints.is_empty() {
      return Ok(Json(ApiResponse::ok(ApiOutPointResult {
        result: None,
        latest_height: height.n(),
        latest_blockhash: blockhash,
      })));
    }

    let mut inscription_digests = Vec::with_capacity(inscriptions_with_satpoints.len());
    for (satpoint, inscription_id, inscription_number) in inscriptions_with_satpoints {
      inscription_digests.push(ApiInscriptionDigest {
        id: inscription_id,
        number: inscription_number,
        location: satpoint,
      });
    }

    let script_pubkey = if let Some(script_pubkey) = script_pubkey {
      script_pubkey
    } else {
      Index::get_tx(outpoint.txid, &rtx, &index)?
        .ok_or(OrdApiError::TransactionNotFound(outpoint.txid))?
        .output
        .into_iter()
        .nth(outpoint.vout.try_into().unwrap())
        .unwrap()
        .script_pubkey
    };

    Ok(Json(ApiResponse::ok(ApiOutPointResult {
      result: Some(ApiOutpointInscriptions {
        txid: outpoint.txid,
        script_pub_key: script_pubkey.to_asm_string(),
        owner: ApiUtxoAddress::from(UtxoAddress::from_script(&script_pubkey, &settings.chain())),
        value,
        inscription_digest: inscription_digests,
      }),
      latest_height: height.n(),
      latest_blockhash: blockhash,
    })))
  })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OutpointsReq {
  pub outpoint: Vec<OutPoint>,
  #[serde(default)]
  pub all_inscription_digests: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiOutPointsResult {
  pub success: bool,
  pub result: HashMap<OutPoint, ApiOutPointResult>,
}

// /api/v1/ord/outpoint/info_batch
/// Retrieve the outpoint information with the specified outpoint.
pub(crate) async fn ord_outpoint_batch(
  Extension(settings): Extension<Arc<Settings>>,
  Extension(index): Extension<Arc<Index>>,
  Json(params): Json<OutpointsReq>,
) -> ApiResult<ApiOutPointsResult> {
  log::info!(
    "rpc: post /api/v1/ord/output/info_batch, outpoint length: {:?}",
    params.outpoint.len()
  );
  let rtx = index.begin_read()?;

  let mut result = HashMap::new();
  let total_now = std::time::Instant::now();
  for (i, outpoint) in params.outpoint.into_iter().enumerate() {
    let now = std::time::Instant::now();
    let Some((latest_height, latest_blockhash)) = Index::latest_block(&rtx)? else {
      return Err(OrdApiError::DataBaseNotReady.into());
    };
    let (inscriptions_with_satpoints, value, script_pubkey) =
      Index::get_inscriptions_on_output_with_satpoints_and_script_pubkey(outpoint, &rtx, &index)?;

    // If there are no inscriptions on the output, return None and parsed block states.
    if inscriptions_with_satpoints.is_empty() {
      result.insert(
        outpoint,
        ApiOutPointResult {
          result: None,
          latest_height: latest_height.n(),
          latest_blockhash: latest_blockhash.clone(),
        },
      );
      continue;
    }

    let mut inscription_digests = Vec::with_capacity(inscriptions_with_satpoints.len());
    let all_inscription_digests = params.all_inscription_digests;
    for (satpoint, inscription_id, inscription_number) in inscriptions_with_satpoints {
      inscription_digests.push(ApiInscriptionDigest {
        id: inscription_id,
        number: inscription_number,
        location: satpoint,
      });
      // 如果不需要全部的 inscription_digests, 则只保存一个
      if !all_inscription_digests {
        break;
      }
    }

    let script_pubkey = if let Some(script_pubkey) = script_pubkey {
      script_pubkey
    } else {
      Index::get_tx(outpoint.txid, &rtx, &index)?
        .ok_or(OrdApiError::TransactionNotFound(outpoint.txid))?
        .output
        .into_iter()
        .nth(outpoint.vout.try_into().unwrap())
        .unwrap()
        .script_pubkey
    };

    let r = ApiOutPointResult {
      result: Some(ApiOutpointInscriptions {
        txid: outpoint.txid,
        script_pub_key: script_pubkey.to_asm_string(),
        owner: ApiUtxoAddress::from(UtxoAddress::from_script(&script_pubkey, &settings.chain())),
        value,
        inscription_digest: inscription_digests,
      }),
      latest_height: latest_height.n(),
      latest_blockhash: latest_blockhash.clone(),
    };
    result.insert(outpoint.clone(), r);
    log::info!(
      "rpc: /api/v1/ord/output/info_batch time used: {:?}, index: {i}",
      now.elapsed()
    );
  }
  log::info!(
    "rpc: /api/v1/ord/output/info_batch, total time used: {:?}",
    total_now.elapsed()
  );
  let r = ApiOutPointsResult {
    success: true,
    result,
  };
  Ok(Json(ApiResponse::ok(r)))
}
