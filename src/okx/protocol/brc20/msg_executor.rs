use super::{
  params::{BIGDECIMAL_TEN, MAXIMUM_SUPPLY, MAX_DECIMAL_WIDTH},
  *,
};
use crate::{
  okx::{
    datastore::{
      brc20::{
        BRC20Error, Balance, Brc20Reader, Brc20ReaderWriter, DeployEvent, Event,
        InscribeTransferEvent, MintEvent, Receipt, Tick, TokenInfo, TransferEvent, TransferableLog,
      },
      ord::OrdReader,
    },
    protocol::{
      brc20::{Message, Mint, Operation},
      context::Context,
    },
  },
  Chain, Result,
};
use anyhow::anyhow;
use bigdecimal::num_bigint::Sign;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionMessage {
  pub(self) txid: Txid,
  pub(self) inscription_id: InscriptionId,
  pub(self) inscription_number: i32,
  pub(self) old_satpoint: SatPoint,
  pub(self) new_satpoint: SatPoint,
  pub(self) from: ScriptKey,
  pub(self) to: Option<ScriptKey>,
  pub(self) op: Operation,
}

impl ExecutionMessage {
  pub fn from_message(context: &mut Context, msg: &Message, chain: Chain) -> Result<Self> {
    Ok(Self {
      txid: msg.txid,
      inscription_id: msg.inscription_id,
      inscription_number: context.get_inscription_number_by_sequence_number(msg.sequence_number)?,
      old_satpoint: msg.old_satpoint,
      new_satpoint: msg
        .new_satpoint
        .ok_or(anyhow!("new satpoint cannot be None"))?,
      from: context.get_script_key_on_satpoint(&msg.old_satpoint, chain)?,
      to: if msg.sat_in_outputs {
        Some(context.get_script_key_on_satpoint(msg.new_satpoint.as_ref().unwrap(), chain)?)
      } else {
        None
      },
      op: msg.op.clone(),
    })
  }
}

pub fn execute(context: &mut Context, msg: &ExecutionMessage) -> Result<Receipt> {
  log::debug!("BRC20 execute message: {:?}", msg);
  let event = match &msg.op {
    Operation::Deploy(deploy) => process_deploy(context, msg, deploy.clone()),
    Operation::Mint { mint, parent } => process_mint(context, msg, mint.clone(), *parent),
    Operation::InscribeTransfer(transfer) => {
      process_inscribe_transfer(context, msg, transfer.clone())
    }
    Operation::Transfer(_) => process_transfer(context, msg),
  };

  let receipt = Receipt {
    inscription_id: msg.inscription_id,
    inscription_number: msg.inscription_number,
    old_satpoint: msg.old_satpoint,
    new_satpoint: msg.new_satpoint,
    from: msg.from.clone(),
    // redirect receiver to sender if transfer to conibase.
    to: msg.to.clone().map_or(msg.from.clone(), |v| v),
    op: msg.op.op_type(),
    result: match event {
      Ok(event) => Ok(event),
      Err(Error::BRC20Error(e)) => Err(e),
      Err(e) => return Err(anyhow!("BRC20 execute exception: {e}")),
    },
  };

  log::debug!("BRC20 message receipt: {:?}", receipt);
  Ok(receipt)
}

fn process_deploy(
  context: &mut Context,
  msg: &ExecutionMessage,
  deploy: Deploy,
) -> Result<Event, Error> {
  // ignore inscribe inscription to coinbase.
  let to_script_key = msg.to.clone().ok_or(BRC20Error::InscribeToCoinbase)?;

  let tick = deploy.tick.parse::<Tick>()?;
  let mut max_supply = deploy.max_supply.clone();
  let mut is_self_mint = false;

  // proposal for issuance self mint token.
  // https://l1f.discourse.group/t/brc-20-proposal-for-issuance-and-burn-enhancements-brc20-ip-1/621
  if tick.self_issuance_tick() {
    if context.chain_conf.blockheight
      < policies::HardForks::self_issuance_activation_height(context.chain_conf.chain)
    {
      return Err(Error::BRC20Error(BRC20Error::SelfIssuanceNotActivated));
    }
    if !deploy.self_mint.unwrap_or_default() {
      return Err(Error::BRC20Error(BRC20Error::SelfIssuanceCheckedFailed));
    }
    if deploy.max_supply == u64::MIN.to_string() {
      max_supply = u64::MAX.to_string();
    }
    is_self_mint = true;
  }

  if let Some(stored_tick_info) = context.get_token_info(&tick).map_err(Error::LedgerError)? {
    return Err(Error::BRC20Error(BRC20Error::DuplicateTick(
      stored_tick_info.tick.to_string(),
    )));
  }

  let dec = Num::from_str(&deploy.decimals.map_or(MAX_DECIMAL_WIDTH.to_string(), |v| v))?
    .checked_to_u8()?;
  if dec > MAX_DECIMAL_WIDTH {
    return Err(Error::BRC20Error(BRC20Error::DecimalsTooLarge(dec)));
  }
  let base = BIGDECIMAL_TEN.checked_powu(u64::from(dec))?;

  let supply = Num::from_str(&max_supply)?;

  if supply.sign() == Sign::NoSign
    || supply > MAXIMUM_SUPPLY.to_owned()
    || supply.scale() > i64::from(dec)
  {
    return Err(Error::BRC20Error(BRC20Error::InvalidSupply(
      supply.to_string(),
    )));
  }

  let limit = Num::from_str(&deploy.mint_limit.map_or(max_supply, |v| v))?;

  if limit.sign() == Sign::NoSign
    || limit > MAXIMUM_SUPPLY.to_owned()
    || limit.scale() > i64::from(dec)
  {
    return Err(Error::BRC20Error(BRC20Error::MintLimitOutOfRange(
      tick.to_lowercase().to_string(),
      limit.to_string(),
    )));
  }

  let supply = supply.checked_mul(&base)?.checked_to_u128()?;
  let limit = limit.checked_mul(&base)?.checked_to_u128()?;

  let new_info = TokenInfo {
    inscription_id: msg.inscription_id,
    inscription_number: msg.inscription_number,
    tick: tick.clone(),
    decimal: dec,
    supply,
    burned_supply: 0u128,
    limit_per_mint: limit,
    minted: 0u128,
    deploy_by: to_script_key,
    is_self_mint,
    deployed_number: context.chain_conf.blockheight,
    latest_mint_number: context.chain_conf.blockheight,
    deployed_timestamp: context.chain_conf.blocktime,
  };
  context
    .insert_token_info(&tick, &new_info)
    .map_err(Error::LedgerError)?;

  Ok(Event::Deploy(DeployEvent {
    supply,
    limit_per_mint: limit,
    decimal: dec,
    tick: new_info.tick,
    self_mint: is_self_mint,
  }))
}

fn process_mint(
  context: &mut Context,
  msg: &ExecutionMessage,
  mint: Mint,
  parent: Option<InscriptionId>,
) -> Result<Event, Error> {
  // ignore inscribe inscription to coinbase.
  let to_script_key = msg.to.clone().ok_or(BRC20Error::InscribeToCoinbase)?;

  let tick = mint.tick.parse::<Tick>()?;

  let tick_info = context
    .get_token_info(&tick)
    .map_err(Error::LedgerError)?
    .ok_or(BRC20Error::TickNotFound(tick.to_string()))?;

  // check if self mint is allowed.
  if tick_info.is_self_mint && !parent.is_some_and(|parent| parent == tick_info.inscription_id) {
    return Err(Error::BRC20Error(BRC20Error::SelfMintPermissionDenied));
  }

  let base = BIGDECIMAL_TEN.checked_powu(u64::from(tick_info.decimal))?;

  let mut amt = Num::from_str(&mint.amount)?;

  if amt.scale() > i64::from(tick_info.decimal) {
    return Err(Error::BRC20Error(BRC20Error::AmountOverflow(
      amt.to_string(),
    )));
  }

  amt = amt.checked_mul(&base)?;
  if amt.sign() == Sign::NoSign {
    return Err(Error::BRC20Error(BRC20Error::InvalidZeroAmount));
  }
  if amt > Into::<Num>::into(tick_info.limit_per_mint) {
    return Err(Error::BRC20Error(BRC20Error::AmountExceedLimit(
      amt.to_string(),
    )));
  }
  let minted = Into::<Num>::into(tick_info.minted);
  let supply = Into::<Num>::into(tick_info.supply);

  if minted >= supply {
    return Err(Error::BRC20Error(BRC20Error::TickMinted(
      tick_info.tick.to_string(),
    )));
  }

  // cut off any excess.
  let mut out_msg = None;
  amt = if amt.checked_add(&minted)? > supply {
    let new = supply.checked_sub(&minted)?;
    out_msg = Some(format!(
      "amt has been cut off to fit the supply! origin: {}, now: {}",
      amt, new
    ));
    new
  } else {
    amt
  };

  // get or initialize user balance.
  let mut balance = context
    .get_balance(&to_script_key, &tick)
    .map_err(Error::LedgerError)?
    .map_or(Balance::new(&tick), |v| v);

  // add amount to available balance.
  balance.overall_balance = Into::<Num>::into(balance.overall_balance)
    .checked_add(&amt)?
    .checked_to_u128()?;

  // store to database.
  context
    .update_token_balance(&to_script_key, balance)
    .map_err(Error::LedgerError)?;

  // update token minted.
  let minted = minted.checked_add(&amt)?.checked_to_u128()?;
  context
    .update_mint_token_info(&tick, minted, context.chain_conf.blockheight)
    .map_err(Error::LedgerError)?;

  Ok(Event::Mint(MintEvent {
    tick: tick_info.tick,
    amount: amt.checked_to_u128()?,
    msg: out_msg,
  }))
}

fn process_inscribe_transfer(
  context: &mut Context,
  msg: &ExecutionMessage,
  transfer: Transfer,
) -> Result<Event, Error> {
  // ignore inscribe inscription to coinbase.
  let to_script_key = msg.to.clone().ok_or(BRC20Error::InscribeToCoinbase)?;

  let tick = transfer.tick.parse::<Tick>()?;

  let token_info = context
    .get_token_info(&tick)
    .map_err(Error::LedgerError)?
    .ok_or(BRC20Error::TickNotFound(tick.to_string()))?;

  let base = BIGDECIMAL_TEN.checked_powu(u64::from(token_info.decimal))?;

  let mut amt = Num::from_str(&transfer.amount)?;

  if amt.scale() > i64::from(token_info.decimal) {
    return Err(Error::BRC20Error(BRC20Error::AmountOverflow(
      amt.to_string(),
    )));
  }

  amt = amt.checked_mul(&base)?;
  if amt.sign() == Sign::NoSign || amt > Into::<Num>::into(token_info.supply) {
    return Err(Error::BRC20Error(BRC20Error::AmountOverflow(
      amt.to_string(),
    )));
  }

  let mut balance = context
    .get_balance(&to_script_key, &tick)
    .map_err(Error::LedgerError)?
    .map_or(Balance::new(&tick), |v| v);

  let overall = Into::<Num>::into(balance.overall_balance);
  let transferable = Into::<Num>::into(balance.transferable_balance);
  let available = overall.checked_sub(&transferable)?;
  if available < amt {
    return Err(Error::BRC20Error(BRC20Error::InsufficientBalance(
      available.to_string(),
      amt.to_string(),
    )));
  }

  balance.transferable_balance = transferable.checked_add(&amt)?.checked_to_u128()?;

  let amt = amt.checked_to_u128()?;
  context
    .update_token_balance(&to_script_key, balance)
    .map_err(Error::LedgerError)?;

  let transferable_asset = TransferableLog {
    inscription_id: msg.inscription_id,
    inscription_number: msg.inscription_number,
    amount: amt,
    tick: token_info.tick.clone(),
    owner: to_script_key,
  };

  context
    .insert_transferable_asset(msg.new_satpoint, &transferable_asset)
    .map_err(Error::LedgerError)?;

  Ok(Event::InscribeTransfer(InscribeTransferEvent {
    tick: transferable_asset.tick,
    amount: amt,
  }))
}

fn process_transfer(context: &mut Context, msg: &ExecutionMessage) -> Result<Event, Error> {
  let transferable = context
    .get_transferable_assets_by_satpoint(&msg.old_satpoint)
    .map_err(Error::LedgerError)?
    .ok_or(BRC20Error::TransferableNotFound(msg.inscription_id))?;

  let amt = Into::<Num>::into(transferable.amount);

  if transferable.owner != msg.from {
    return Err(Error::BRC20Error(BRC20Error::TransferableOwnerNotMatch(
      msg.inscription_id,
    )));
  }

  let tick = transferable.tick;

  let token_info = context
    .get_token_info(&tick)
    .map_err(Error::LedgerError)?
    .ok_or(BRC20Error::TickNotFound(tick.to_string()))?;

  // update from key balance.
  let mut from_balance = context
    .get_balance(&msg.from, &tick)
    .map_err(Error::LedgerError)?
    .map_or(Balance::new(&tick), |v| v);

  let from_overall = Into::<Num>::into(from_balance.overall_balance);
  let from_transferable = Into::<Num>::into(from_balance.transferable_balance);

  let from_overall = from_overall.checked_sub(&amt)?.checked_to_u128()?;
  let from_transferable = from_transferable.checked_sub(&amt)?.checked_to_u128()?;

  from_balance.overall_balance = from_overall;
  from_balance.transferable_balance = from_transferable;

  context
    .update_token_balance(&msg.from, from_balance)
    .map_err(Error::LedgerError)?;

  // redirect receiver to sender if transfer to conibase.
  let mut out_msg = None;

  let to_script_key = if msg.to.clone().is_none() {
    out_msg =
      Some("redirect receiver to sender, reason: transfer inscription to coinbase".to_string());
    msg.from.clone()
  } else {
    msg.to.clone().unwrap()
  };

  // update to key balance.
  let mut to_balance = context
    .get_balance(&to_script_key, &tick)
    .map_err(Error::LedgerError)?
    .map_or(Balance::new(&tick), |v| v);

  let to_overall = Into::<Num>::into(to_balance.overall_balance);
  to_balance.overall_balance = to_overall.checked_add(&amt)?.checked_to_u128()?;

  context
    .update_token_balance(&to_script_key, to_balance)
    .map_err(Error::LedgerError)?;

  context
    .remove_transferable_asset(msg.old_satpoint)
    .map_err(Error::LedgerError)?;

  // update burned supply if transfer to op_return.
  match to_script_key {
    ScriptKey::ScriptHash { is_op_return, .. } if is_op_return => {
      let burned_amt = Into::<Num>::into(token_info.burned_supply)
        .checked_add(&amt)?
        .checked_to_u128()?;
      context
        .update_burned_token_info(&tick, burned_amt)
        .map_err(Error::LedgerError)?;
      out_msg = Some(format!(
        "transfer to op_return, burned supply increased: {}",
        amt
      ));
    }
    _ => (),
  }

  Ok(Event::Transfer(TransferEvent {
    msg: out_msg,
    tick: token_info.tick,
    amount: amt.checked_to_u128()?,
  }))
}
