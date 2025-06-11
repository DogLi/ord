use std::cmp::Ordering;

use super::*;
use crate::okx::brc20::{
  brc20_decimal::Brc20Decimal, entry::BRC20ModuleTokenBalance, utils::get_valid_unique_lower_ticker,
};

impl BRC20ExecutionMessage {
  pub(super) fn execute_inscribe_withdraw(
    &self,
    context: &mut TableContext,
  ) -> Result<BRC20Receipt, ExecutionError> {
    log::info!(
      "brc20swap execute_inscribe_withdraw: {:?}, inscription_id: {:?}, sender: {:?}, receiver: {:?}",
      self.txid,
      self.inscription_id,
      self.sender,
      self.receiver,
    );

    let BRC20Operation::Withdraw(withdraw) = &self.operation else {
      log::debug!(
        "brc20swap error execute_inscribe_withdraw unreachable: {:?}, inscription_id: {:?}",
        self.txid,
        self.inscription_id
      );
      unreachable!()
    };
    log::debug!("brc20swap execute_inscribe_withdraw: {:?}", withdraw);

    // if receiver no value, return error
    let Some(address) = self.receiver.as_ref() else {
      log::debug!(
        "brc20swap error execute_inscribe_withdraw receiver no value: {:?}, inscription_id: {:?}, sender: {:?}, receiver: {:?}",
        self.txid,
        self.inscription_id,
        self.sender,
        self.receiver
      );
      return Err(ExecutionError::ExecutionFailed(BRC20Error::InvalidAddress()));
    };

    // only accept lower case module_id
    if withdraw.module.to_lowercase() != withdraw.module {
      log::debug!(
        "brc20swap error execute_inscribe_withdraw module_id invalid: {:?}",
        withdraw.module
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::ModuleIDInvalid(withdraw.module.clone()),
      ));
    }

    // the module exists?
    let module_info = match context.load_brc20_module_info(&withdraw.module)? {
      Some(module_info) => module_info,
      None => {
        log::debug!(
          "brc20swap error execute_inscribe_withdraw module_id not exists: {:?}",
          withdraw.module
        );
        return Err(ExecutionError::ExecutionFailed(
          BRC20Error::ModuleNotExists(withdraw.module.clone()),
        ));
      }
    };

    let unique_lower_ticker =
      BRC20Ticker::from_str(&get_valid_unique_lower_ticker(&withdraw.tick)?).map_err(|e| {
        log::debug!(
          "brc20swap error execute_inscribe_withdraw ticker invalid: {:?}",
          withdraw.tick
        );
        ExecutionError::ExecutionFailed(BRC20Error::TickerInvalid(e.to_string()))
      })?;

    // the ticker exists?
    let ticker_info = match context.load_brc20_ticker_info(&unique_lower_ticker)? {
      Some(ticker_info) => ticker_info,
      None => {
        log::debug!(
          "brc20swap error execute_inscribe_withdraw ticker not found: {:?}",
          unique_lower_ticker
        );
        return Err(ExecutionError::ExecutionFailed(BRC20Error::TickerNotFound(
          unique_lower_ticker.to_string(),
        )));
      }
    };

    let amount =
      Brc20Decimal::from_str_with_scale(&withdraw.amt, ticker_info.decimals).map_err(|_| {
        log::debug!(
          "brc20swap error execute_inscribe_withdraw amount invalid: {:?}, withdraw.amt: {:?}",
          withdraw.amt.clone(),
          withdraw.amt
        );
        ExecutionError::ExecutionFailed(BRC20Error::InvalidAmountFormat(withdraw.amt.clone()))
      })?;

    // is amount valid?
    let total_supply =
      Brc20Decimal::from_u128(ticker_info.total_supply, ticker_info.decimals).unwrap();
    if amount.sign() < 0 || amount.cmp(&total_supply) == Ordering::Greater {
      log::debug!(
        "brc20swap error execute_inscribe_withdraw amount invalid: {:?}, total_supply: {:?}",
        amount.clone(),
        total_supply.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InvalidWithdrawAmount(amount),
      ));
    }

    let mut address_module_balance = match context.load_brc20_module_address_token_balance(
      &address.clone(),
      &module_info.id,
      &unique_lower_ticker,
    )? {
      Some(balance) => balance,
      None => BRC20ModuleTokenBalance::new_with_scale(ticker_info.decimals),
    };

    address_module_balance.pending_withdrawal_amount =
      address_module_balance.pending_withdrawal_amount + amount.clone();

    context.insert_brc20_module_inscribe_withdraw(self.new_satpoint, withdraw.clone())?;
    context.update_brc20_module_address_token_balance(
      &address.clone(),
      &module_info.id,
      &unique_lower_ticker,
      address_module_balance,
    )?;

    log::debug!("brc20swap execute_inscribe_withdraw finished: {:?}", self);
    Ok(BRC20Receipt {
      inscription_id: self.inscription_id,
      sequence_number: self.sequence_number,
      inscription_number: 0,
      old_satpoint: self.old_satpoint,
      new_satpoint: self.new_satpoint,
      op_type: BRC20OpType::Withdraw,
      sender: self.sender.clone(),
      receiver: self
        .receiver
        .as_ref()
        .cloned()
        .unwrap_or_else(|| self.sender.clone()),
      result: Ok(BRC20Event::InscribeWithdraw(event::InscribeWithdrawEvent {
        ticker: unique_lower_ticker.clone(),
        amount: amount.clone().to_string(),
        module: module_info.id.clone(),
      })),
    })
  }

  pub(super) fn execute_transfer_withdraw(
    &self,
    index: &Index,
    context: &mut TableContext,
  ) -> Result<BRC20Receipt, ExecutionError> {
    log::info!(
      "brc20swap execute_transfer_withdraw: {:?}, inscription_id: {:?}, sender: {:?}, receiver: {:?}",
      self.txid,
      self.inscription_id,
      self.sender,
      self.receiver,
    );

    let BRC20Operation::TransferWithdraw(withdraw) = &self.operation else {
      log::debug!(
        "brc20swap error execute_transfer_withdraw unreachable: {:?}, inscription_id: {:?}",
        self.txid,
        self.inscription_id
      );
      unreachable!()
    };
    log::debug!("brc20swap execute_transfer_withdraw: {:?}", withdraw);

    match context.load_brc20_module_inscribe_withdraw(self.old_satpoint)? {
      Some(withdraw) => withdraw,
      None => {
        log::debug!(
          "brc20swap error execute_transfer_withdraw inscribe_withdraw not found: {:?}, inscription_id: {:?}",
          self.old_satpoint,
          self.inscription_id
        );
        return Err(ExecutionError::ExecutionFailed(
          BRC20Error::InscribeWithdrawNotFound(self.old_satpoint.to_string()),
        ));
      }
    };
    match context.remove_brc20_module_inscribe_withdraw(self.old_satpoint) {
      Ok(_) => (),
      Err(e) => {
        log::debug!(
          "brc20swap error execute_transfer_withdraw remove_brc20_module_inscribe_withdraw error: {:?}",
          e
        );
        return Err(ExecutionError::ExecutionFailed(
          BRC20Error::RemoveInscribeWithdrawError(self.old_satpoint.to_string()),
        ));
      }
    }

    let unique_lower_ticker =
      BRC20Ticker::from_str(&get_valid_unique_lower_ticker(&withdraw.tick)?)
        .map_err(|e| ExecutionError::ExecutionFailed(BRC20Error::TickerInvalid(e.to_string())))?;
    let ticker_info = match context.load_brc20_ticker_info(&unique_lower_ticker)? {
      Some(ticker_info) => ticker_info,
      None => {
        log::debug!(
          "brc20swap error execute_transfer_withdraw ticker not found: {:?}, ticker: {:?}, address: {:?}",
          withdraw.module,
          unique_lower_ticker,
          self.sender
        );
        return Err(ExecutionError::ExecutionFailed(BRC20Error::TickerNotFound(
          unique_lower_ticker.to_string(),
        )));
      }
    };
    let amount =
      Brc20Decimal::from_str_with_scale(&withdraw.amt, ticker_info.decimals).map_err(|_| {
        ExecutionError::ExecutionFailed(BRC20Error::InvalidAmountFormat(withdraw.amt.clone()))
      })?;
    log::debug!("brc20swap execute_transfer_withdraw amount: {:?}", amount);

    // load and update brc20 module balance
    let mut sender_module_balance = match context.load_brc20_module_address_token_balance(
      &self.sender,
      &withdraw.module,
      &unique_lower_ticker,
    )? {
      Some(balance) => balance,
      None => {
        log::debug!(
          "brc20swap error execute_transfer_withdraw module balance not exists: {:?}, ticker: {:?}, address: {:?} ",
          withdraw.module,
          unique_lower_ticker,
          self.sender
        );
        return Err(ExecutionError::ExecutionFailed(
          BRC20Error::ModuleBalanceNotExists(
            withdraw.module.clone(),
            unique_lower_ticker.to_string(),
          ),
        ));
      }
    };

    log::debug!(
      "brc20swap execute_transfer_withdraw sender_module_balance.pending_withdrawal_amount: {:?}",
      sender_module_balance.pending_withdrawal_amount.clone()
    );
    sender_module_balance.pending_withdrawal_amount =
      sender_module_balance.pending_withdrawal_amount - amount.clone();
    context.update_brc20_module_address_token_balance(
      &self.sender,
      &withdraw.module,
      &unique_lower_ticker,
      sender_module_balance.clone(),
    )?;

    if sender_module_balance.available_balance.clone() < amount.clone() {
      log::debug!(
        "brc20swap error execute_transfer_withdraw amount invalid: {:?}, available_balance: {:?}",
        amount.clone(),
        sender_module_balance.available_balance.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InsufficientWithdrawAmount(
          amount.clone(),
          sender_module_balance.available_balance.clone(),
        ),
      ));
    }
    log::debug!(
      "brc20swap execute_transfer_withdraw sender_module_balance.available_balance: {:?}",
      sender_module_balance.available_balance.clone()
    );

    sender_module_balance.available_balance =
      sender_module_balance.available_balance - amount.clone();
    context.update_brc20_module_address_token_balance(
      &self.sender,
      &withdraw.module,
      &unique_lower_ticker,
      sender_module_balance.clone(),
    )?;

    let receiver = self.receiver.as_ref().unwrap_or(&self.sender);
    log::debug!(
      "brc20swap execute_transfer_withdraw receiver: {:?}",
      receiver
    );
    // load and update brc20 ticker balance
    let mut receiver_brc20_balance =
      match context.load_brc20_balance(&receiver, &unique_lower_ticker)? {
        Some(balance) => balance,
        None => BRC20Balance {
          ticker: unique_lower_ticker.clone(),
          total: 0,
          available: 0,
        },
      };
    let (amount_value, _) = amount.to_u128();
    log::debug!(
      "brc20swap execute_transfer_withdraw amount_value: {:?}, receiver_brc20_balance.total: {:?}, receiver_brc20_balance.available: {:?}",
      amount_value,
      receiver_brc20_balance.total,
      receiver_brc20_balance.available
    );
    receiver_brc20_balance.total += amount_value;
    receiver_brc20_balance.available += amount_value;
    context.update_brc20_balance(&receiver, &unique_lower_ticker, receiver_brc20_balance)?;

    // burn
    let receiver_pk_script = receiver.to_script(&index.chain());
    if receiver_pk_script.is_op_return() {
      let mut ticker_info = context
        .load_brc20_ticker_info(&unique_lower_ticker)?
        .ok_or(ExecutionError::ExecutionFailed(BRC20Error::TickerNotFound(
          unique_lower_ticker.to_string(),
        )))?;
      ticker_info.burned += amount_value;
      context.update_brc20_ticker_info(&unique_lower_ticker, ticker_info)?;
    }

    log::debug!("brc20swap execute_transfer_withdraw finished: {:?}", self);
    Ok(BRC20Receipt {
      inscription_id: self.inscription_id,
      sequence_number: self.sequence_number,
      inscription_number: 0,
      old_satpoint: self.old_satpoint,
      new_satpoint: self.new_satpoint,
      op_type: BRC20OpType::TransferWithdraw,
      sender: self.sender.clone(),
      receiver: self
        .receiver
        .as_ref()
        .cloned()
        .unwrap_or_else(|| self.sender.clone()),
      result: Ok(BRC20Event::TransferWithdraw(event::TransferWithdrawEvent {
        ticker: unique_lower_ticker.clone(),
        amount: amount.clone().to_string(),
        module: withdraw.module.clone(),
      })),
    })
  }
}
