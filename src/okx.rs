use super::*;
use crate::okx::brc20::event::{BRC20Event, BRC20OpType, UnisatSwapEvent};
use crate::okx::brc20::{BRC20Balance, BRC20Error, BRC20Ticker, FixedPoint};
use crate::okx::uniswap::client::WithdrawHistory;
use crate::{
  index::{
    bundle_message::{BundleMessage, InscriptionAction, SubType},
    event::{Action, OkxInscriptionEvent},
    BlockData,
  },
  metrics::MetricsExt,
};
use brc20::{BRC20ExecutionMessage, BRC20Receipt};
use context::TableContext;
use entry::CollectionType;
use std::collections::HashMap;

pub(crate) mod bitmap;
pub(crate) mod brc20;
pub(crate) mod btc_domain;
mod composite_key;
pub(crate) mod context;
pub(crate) mod entry;
pub mod uniswap;
mod utxo_address;

pub(crate) use self::{
  composite_key::{AddressEndpoint, AddressTickerKey},
  utxo_address::{UtxoAddress, UtxoAddressRef},
};

pub(crate) struct OkxUpdater {
  pub(crate) height: u32,
  pub(crate) timestamp: u32,
}

impl OkxUpdater {
  pub(crate) fn index_block_bundle_messages(
    &mut self,
    context: &mut TableContext,
    index: &Index,
    block_data: &BlockData,
    mut bundle_messages_map: HashMap<Txid, Vec<BundleMessage>>,
  ) -> Result<()> {
    let start_time = Instant::now();
    let mut total_inscription_receipts = 0;
    let mut total_brc20_receipts = 0;
    let mut total_bitmap_messages = 0;
    let mut total_btc_domain_messages = 0;

    log::info!(
      "[OKX] Starting to index block {} at {}, transaction_count: {}, bundle_message_count: {})",
      self.height,
      timestamp(self.timestamp.into()),
      block_data.txdata.len(),
      bundle_messages_map.len()
    );

    let mut inscription_id_list = vec![];
    for (_, msg_list) in bundle_messages_map.iter() {
      let m = msg_list.iter().map(|m| m.inscription_id.clone());
      inscription_id_list.extend(m);
    }

    for (_tx_index, (_transaction, txid)) in block_data
      .txdata
      .iter()
      .enumerate()
      .skip(1)
      .chain(block_data.txdata.iter().enumerate().take(1))
    {
      let withdraw_history_list: Vec<_> = block_data
        .withdraw_histories
        .iter()
        .filter(|history| &history.txid == txid)
        .collect();
      // 检查 withdraw history 的 inscription_id 是否包含在链上
      for withdraw_history in withdraw_history_list.iter() {
        if !inscription_id_list.contains(&withdraw_history.inscription_id) {
          log::error!(
            "can't find withdraw history inscription id {:?} in op list",
            withdraw_history.inscription_id
          );
          bail!(
            "can't find withdraw history inscription id {:?}",
            withdraw_history.inscription_id
          );
        }
      }
      if let Some(transaction_bundle_messages) = bundle_messages_map.remove(txid) {
        let (brc20_receipts, bitmap_message_count, btc_domain_message_count) = self
          .process_bundle_messages(
            context,
            index,
            &transaction_bundle_messages,
            withdraw_history_list,
          )?;
        total_brc20_receipts += brc20_receipts.len();
        total_bitmap_messages += bitmap_message_count;
        total_btc_domain_messages += btc_domain_message_count;

        if !brc20_receipts.is_empty() {
          let brc20_receipts_count = brc20_receipts.len();
          let start_insert_time = Instant::now();

          let sequence_number_list = brc20_receipts
            .iter()
            .map(|receipt| receipt.sequence_number)
            .collect::<HashSet<_>>();

          for sequence_number in sequence_number_list {
            context
              .insert_sequence_number_to_collection_type(sequence_number, CollectionType::BRC20)?;
          }

          context.insert_brc20_tx_receipts(txid, brc20_receipts)?;
          log::debug!(
            "[OKX] Saved {} BRC20 receipts for transaction {} in {} ms",
            brc20_receipts_count,
            txid,
            (Instant::now() - start_insert_time).as_millis()
          );
        }
        if index.has_inscription_receipts() {
          let transaction_bundle_messages_count = transaction_bundle_messages.len();
          total_inscription_receipts += transaction_bundle_messages_count;
          let inscription_receipts = transaction_bundle_messages
            .into_iter()
            .map(Into::into)
            .collect();
          let start_insert_time = Instant::now();
          context.insert_inscription_tx_receipts(txid, inscription_receipts)?;
          log::debug!(
            "[OKX] Saved {} inscription receipts for transaction {} in {} ms",
            transaction_bundle_messages_count,
            txid,
            (Instant::now() - start_insert_time).as_millis()
          );
        }
      }
    }

    if index.has_brc20_index() {
      index
        .metrics
        .increment_brc20_event_count(u32::try_from(total_brc20_receipts).unwrap());
    }
    if index.has_inscription_receipts() {
      index
        .metrics
        .increment_inscription_event_count(u32::try_from(total_inscription_receipts).unwrap());
    }

    log::info!(
            "[OKX] Finished indexing block {} {{ total_inscriptions: {}, total_brc20: {}, total_bitmaps: {}, total_btc_domains: {} }} in {} ms",
            self.height,
            total_inscription_receipts,
            total_brc20_receipts,
            total_bitmap_messages,
            total_btc_domain_messages,
            (Instant::now() - start_time).as_millis(),
        );

    Ok(())
  }

  fn process_bundle_messages(
    &self,
    context: &mut TableContext,
    index: &Index,
    bundle_messages: &[BundleMessage],
    withdraw_history_list: Vec<&WithdrawHistory>,
  ) -> Result<(Vec<BRC20Receipt>, usize, usize)> {
    let mut brc20_execution_receipts = Vec::new();
    let mut bitmap_message_count = 0;
    let mut btc_domain_message_count = 0;

    for bundle_message in bundle_messages.iter() {
      // process brc20 operation
      if index.has_brc20_index() {
        if let Some(brc20_execution_message) =
          BRC20ExecutionMessage::new_from_bundle_message(bundle_message, context)?
        {
          if let Ok(receipt) = brc20_execution_message.execute(context, self.height, self.timestamp)
          {
            brc20_execution_receipts.push(receipt);
          }
          continue;
        }
      }

      // process uniswap withdraw history
      for withdraw_history in withdraw_history_list.iter() {
        log::info!(
          "execute unisat withdraw history, the txid is: {:?}",
          bundle_message.txid
        );
        // get the sequence_number
        let sequence_number = bundle_messages
          .iter()
          .find(|i| i.txid == withdraw_history.txid)
          .map(|i| i.sequence_number)
          .unwrap_or(0);
        let receipt = self
          .execute_unisat_swap(sequence_number, context, withdraw_history)
          .context("execute unisat swap failed")?;
        brc20_execution_receipts.push(receipt);
      }

      // process bitmap operation
      if index.has_bitmap_index() {
        if let InscriptionAction::Created {
          sub_type: Some(SubType::Bitmap(bitmap_operation)),
          ..
        } = &bundle_message.inscription_action
        {
          bitmap_message_count += 1;
          bitmap_operation.execute(
            context,
            bundle_message.sequence_number,
            bundle_message.inscription_id,
            self.height,
          )?;
        }
      }

      // process btc domain operation
      if index.has_btc_domain_index() {
        if let InscriptionAction::Created {
          sub_type: Some(SubType::BtcDomain(btc_domain)),
          ..
        } = &bundle_message.inscription_action
        {
          btc_domain_message_count += 1;
          btc_domain.execute(
            context,
            bundle_message.sequence_number,
            bundle_message.inscription_id,
          )?;
        }
      }
    }

    Ok((
      brc20_execution_receipts,
      bitmap_message_count,
      btc_domain_message_count,
    ))
  }

  fn execute_unisat_swap(
    &self,
    sequence_number: u32,
    context: &mut TableContext,
    withdraw_history: &WithdrawHistory,
  ) -> Result<BRC20Receipt> {
    let event = process_withdraw_history(context, withdraw_history);
    let receipt = BRC20Receipt {
      inscription_id: withdraw_history.inscription_id,
      sequence_number,
      inscription_number: withdraw_history.inscription_number,
      old_satpoint: withdraw_history.old_sat_point,
      new_satpoint: withdraw_history.new_sat_point,
      sender: UtxoAddress::from_address(withdraw_history.from_address()?),
      receiver: UtxoAddress::from_address(withdraw_history.to_address()?),
      op_type: BRC20OpType::UnisatSwapWithdraw,
      result: event,
    };
    Ok(receipt)
  }
}

fn process_withdraw_history(
  context: &mut TableContext,
  withdraw_history: &WithdrawHistory,
) -> Result<BRC20Event, BRC20Error> {
  //  from/to 检查
  withdraw_history.from_address().map_err(|_| {
    log::error!(
      "invalid from address in withdraw history: {:?}",
      withdraw_history
    );
    BRC20Error::InvalidAddress(withdraw_history.from.clone())
  })?;
  let to_address = withdraw_history.to_address().map_err(|_| {
    log::error!(
      "invalid to_address in withdraw history: {:?}",
      withdraw_history
    );
    BRC20Error::InvalidAddress(withdraw_history.to.clone())
  })?;
  let to_address = UtxoAddress::from_address(to_address);

  // 获取精度，乘以精度
  let ticker =
    BRC20Ticker::from_str(&withdraw_history.data.tick).map_err(BRC20Error::TickerParse)?;
  let ticker_info = context
    .load_brc20_ticker_info(&ticker)
    .map_err(|e| BRC20Error::DBError(e.to_string()))?
    .ok_or(BRC20Error::TickerNotFound(
      withdraw_history.data.tick.clone(),
    ))?;

  let amount =
    FixedPoint::new_from_str(&withdraw_history.data.readable_amount, ticker_info.decimals)
      .map_err(BRC20Error::NumericError)?;

  // update to key balance.
  let mut to_balance = context
    .load_brc20_balance(&to_address, &ticker)
    .map_err(|e| BRC20Error::DBError(e.to_string()))?
    .unwrap_or(BRC20Balance::new_with_ticker(&ticker));

  let to_overall =
    FixedPoint::new(to_balance.total, ticker_info.decimals).map_err(BRC20Error::NumericError)?;
  to_balance.total = (to_overall + amount).to_u128_and_scale().0;
  log::info!("process withdraw history at height {}, txid:{:?}, add {}{} to address {:?}, {:?} + {:?} = {:?}",
    withdraw_history.height,
    withdraw_history.txid,
    withdraw_history.data.readable_amount,
    withdraw_history.data.tick,
    to_address,
    to_overall,
    amount.to_string(),
    to_balance.total,
  );

  context
    .update_brc20_balance(&to_address, &ticker, to_balance)
    .map_err(|e| BRC20Error::DBError(e.to_string()))?;

  // update burned supply if transfer to op_return.
  Ok(BRC20Event::UnisatSwap(UnisatSwapEvent {
    tick: ticker,
    amount: amount.to_u128_and_scale().0,
  }))
}
