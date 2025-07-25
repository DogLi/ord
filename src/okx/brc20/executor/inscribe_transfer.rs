use std::ops::Add;

use crate::okx::{
  brc20::{brc20_decimal::Brc20Decimal, entry::BRC20ModuleTokenBalance},
  utils::get_module_from_script,
};

use super::*;

impl BRC20ExecutionMessage {
  pub(super) fn execute_inscribe_transfer(
    &self,
    index: &Index,
    context: &mut TableContext,
  ) -> Result<BRC20Receipt, ExecutionError> {
    let BRC20Operation::InscribeTransfer{ signer, transfer } = &self.operation else {
      log::debug!(
        "brc20swap execute_inscribe_transfer unreachable: {:?}, inscription_id: {:?}",
        self.txid,
        self.inscription_id
      );
      unreachable!()
    };

    let ticker = BRC20Ticker::from_str(&transfer.tick).map_err(BRC20Error::TickerParse)?;

    // load ticker info, ensure the ticker is deployed
    let mut ticker_info = context
      .load_brc20_ticker_info(&ticker)?
      .ok_or(BRC20Error::TickerNotFound(transfer.tick.clone()))?;

    let ticker = ticker_info.ticker.clone();
    let decimals = ticker_info.clone().decimals;
    let total_supply = ticker_info.clone().total_supply;

    let amt = FixedPoint::new_from_str(&transfer.amount, decimals)
      .map_err(BRC20Error::NumericError)?;
    if amt.is_zero()
      || amt > FixedPoint::new_unchecked(total_supply, decimals)
    {
      log::debug!(
        "brc20swap execute_inscribe_transfer amount invalid: {:?}, amount: {:?}",
        self.txid,
        amt
      );
      return Err(ExecutionError::ExecutionFailed(BRC20Error::InvalidAmount(
        amt,
      )));
    }

    let mut sender_or_legacy = self.sender.clone();
    let receiver = self.receiver.clone().unwrap();
    let mut sender = receiver.clone();
    let mut sender_balance = context
      .load_brc20_balance(&sender, &ticker)?
      .unwrap_or(BRC20Balance::new_with_ticker(&ticker));

    let mut receiver_balance: Option<BRC20Balance> = None;
    if let Some(signer) = signer.clone() {
      sender_or_legacy = signer.clone();
      if signer != sender {
        receiver_balance = Some(sender_balance);

        sender = signer;
        sender_balance = context
          .load_brc20_balance(&sender, &ticker)?
          .unwrap_or(BRC20Balance::new_with_ticker(&ticker));
      }
    }

    let available = FixedPoint::new_unchecked(sender_balance.available, decimals);
    sender_balance.available = available
      .checked_sub(amt)
      .ok_or(ExecutionError::ExecutionFailed(
        BRC20Error::InsufficientBalance(available, amt),
      ))?
      .to_u128_and_scale()
      .0;

    let amount = amt.to_u128_and_scale().0;
    if let Some(mut receiver_balance) = receiver_balance {
      sender_balance.total = sender_balance
        .total
        .checked_sub(amount)
        .expect("Subtraction overflow");

      receiver_balance.total = receiver_balance
        .total
        .checked_add(amount)
        .expect("Addition overflow");

      context.update_brc20_balance(&receiver, &ticker, receiver_balance)?;

      // burn
      let burned = receiver.op_return();
      if burned {
        ticker_info.burned = ticker_info
          .burned
          .checked_add(amount)
          .expect("Addition overflow");

        context.update_brc20_ticker_info(&ticker, ticker_info.clone())?;
      }

      // brc20 swap
      let script_buf = receiver.to_script(&index.chain());
      let module_id = get_module_from_script(&script_buf);
      match module_id {
        Ok(module_id_str) => {
          log::info!(
            "brc20swap transfer: {:?}, inscription_id: {:?}",
            self.txid,
            self.inscription_id
          );
          let is_deploy = context.is_module_already_deployed(&module_id_str)?;
          if is_deploy {
            let module_receiver = sender.clone();
            let mut receiver_module_balance = context
              .load_brc20_module_address_token_balance(&module_receiver, &module_id_str, &ticker)?
              .unwrap_or(BRC20ModuleTokenBalance::new_with_scale(decimals));
            receiver_module_balance.swap_account_balance = receiver_module_balance
              .swap_account_balance
              .add(Brc20Decimal::from_u128(amount, decimals).unwrap());
            context.update_brc20_module_address_token_balance(
              &module_receiver,
              &module_id_str,
              &ticker,
              receiver_module_balance,
            )?;
          }
        }
        Err(_) => {}
      }

    }

    context.update_brc20_balance(&sender, &ticker, sender_balance)?;

    let transferring_asset = BRC20TransferAsset {
      ticker: ticker.clone(),
      amount: amount,
      owner: receiver.clone(),
      sequence_number: self.sequence_number,
      inscription_number: 0,
      inscription_id: self.inscription_id,
    };

    context.insert_brc20_transferring_asset(
      &receiver,
      &ticker,
      self.new_satpoint,
      transferring_asset,
    )?;

    log::debug!("brc20swap execute_inscribe_transfer finished: {:?}", self);
    Ok(BRC20Receipt {
      inscription_id: self.inscription_id,
      sequence_number: self.sequence_number,
      inscription_number: 0,
      old_satpoint: self.old_satpoint,
      new_satpoint: self.new_satpoint,
      sender: sender_or_legacy,
      receiver: receiver,
      op_type: BRC20OpType::InscribeTransfer,
      result: Ok(BRC20Event::InscribeTransfer(InscribeTransferEvent {
        ticker,
        amount: amount,
      })),
    })
  }
}
