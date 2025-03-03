use std::cmp::Ordering;

use super::*;
use crate::okx::brc20::{
  brc20_decimal::Brc20Decimal,
  entry::{BRC20ModuleTokenBalance, InscribeWithdraw},
  utils::get_valid_unique_lower_ticker,
};

impl BRC20ExecutionMessage {
  pub(super) fn execute_inscribe_withdraw(
    &self,
    context: &mut TableContext,
  ) -> Result<BRC20Receipt, ExecutionError> {
    log::info!(
      "brc20swap execute_inscribe_withdraw: {:?}, inscription_id: {:?}",
      self.txid,
      self.inscription_id
    );

    let BRC20Operation::Withdraw(withdraw) = &self.operation else {
      log::debug!(
        "brc20swap execute_inscribe_withdraw unreachable: {:?}, inscription_id: {:?}",
        self.txid,
        self.inscription_id
      );
      unreachable!()
    };

    // only accept lower case module_id
    if withdraw.module.to_lowercase() != withdraw.module {
      log::debug!(
        "brc20swap execute_inscribe_withdraw module_id invalid: {:?}",
        withdraw.module
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::ModuleIDInvalid(withdraw.module.clone()),
      ));
    }

    // the module exists?
    let module_info =
      context
        .load_brc20_module_info(&withdraw.module)?
        .ok_or(ExecutionError::ExecutionFailed(
          BRC20Error::ModuleNotExists(withdraw.module.clone()),
        ))?;

    let unique_lower_ticker =
      BRC20Ticker::from_str(&get_valid_unique_lower_ticker(&withdraw.tick)?)
        .map_err(|e| ExecutionError::ExecutionFailed(BRC20Error::TickerInvalid(e.to_string())))?;

    // the ticker exists?
    let ticker_info = context
      .load_brc20_ticker_info(&unique_lower_ticker)?
      .ok_or(ExecutionError::ExecutionFailed(BRC20Error::TickerNotFound(
        unique_lower_ticker.to_string(),
      )))?;

    let amount = Brc20Decimal::from_str(&withdraw.amt).map_err(|_| {
      ExecutionError::ExecutionFailed(BRC20Error::InvalidAmountFormat(withdraw.amt.clone()))
    })?;

    // is amount valid?
    let total_supply = Brc20Decimal::from_u128(ticker_info.total_supply, 0).unwrap();
    if amount.sign() < 0 || amount.cmp(&total_supply) == Ordering::Greater {
      log::debug!(
        "brc20swap execute_inscribe_withdraw amount invalid: {:?}, total_supply: {:?}",
        amount.clone(),
        total_supply.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InvalidWithdrawAmount(amount),
      ));
    }

    let mut module_balance = match context.load_brc20_module_address_token_balance(
      &self.sender,
      &module_info.id,
      &unique_lower_ticker,
    )? {
      Some(balance) => balance,
      None => BRC20ModuleTokenBalance::new(),
    };

    if amount.cmp(&module_balance.available_balance) == Ordering::Greater {
      log::debug!(
        "brc20swap execute_inscribe_withdraw amount invalid: {:?}, available_balance: {:?}",
        amount.clone(),
        module_balance.available_balance.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InsufficientWithdrawAmount(amount, module_balance.available_balance),
      ));
    }
    module_balance.pending_withdrawal_amount =
      module_balance.pending_withdrawal_amount + amount.clone();

    let withdraw = InscribeWithdraw {
      sender: self.sender.clone(),
      module_id: module_info.id.clone(),
      ticker: unique_lower_ticker.to_lowercase(),
      amount: amount.clone(),
    };
    context.insert_brc20_module_withdraw(self.new_satpoint, withdraw)?;
    context.update_brc20_module_address_token_balance(
      &self.sender,
      &module_info.id,
      &unique_lower_ticker,
      module_balance,
    )?;

    Ok(BRC20Receipt {
      inscription_id: self.inscription_id,
      sequence_number: self.sequence_number,
      inscription_number: self.inscription_number,
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
      "brc20swap execute_transfer_withdraw: {:?}, inscription_id: {:?}",
      self.txid,
      self.inscription_id
    );

    //TODO: need to update tables in a transaction

    let BRC20Operation::Withdraw(withdraw) = &self.operation else {
      log::debug!(
        "brc20swap execute_transfer_withdraw unreachable: {:?}, inscription_id: {:?}",
        self.txid,
        self.inscription_id
      );
      unreachable!()
    };

    let inscribe_withdraw = match context.load_brc20_module_inscribe_withdraw(self.old_satpoint)? {
      Some(withdraw) => withdraw,
      None => {
        log::debug!(
          "brc20swap execute_transfer_withdraw inscribe_withdraw not found: {:?}, inscription_id: {:?}",
          self.old_satpoint,
          self.inscription_id
        );
        return Err(ExecutionError::ExecutionFailed(
          BRC20Error::InscribeWithdrawNotFound(self.old_satpoint.to_string()),
        ));
      }
    };

    let unique_lower_ticker =
      BRC20Ticker::from_str(&get_valid_unique_lower_ticker(&withdraw.tick)?)
        .map_err(|e| ExecutionError::ExecutionFailed(BRC20Error::TickerInvalid(e.to_string())))?;

    // load and update brc20 module balance
    let mut module_balance = match context.load_brc20_module_address_token_balance(
      &self.sender,
      &withdraw.module,
      &unique_lower_ticker,
    )? {
      Some(balance) => balance,
      None => {
        return Err(ExecutionError::ExecutionFailed(
          BRC20Error::ModuleBalanceNotExists(
            withdraw.module.clone(),
            unique_lower_ticker.to_string(),
          ),
        ));
      }
    };

    module_balance.pending_withdrawal_amount =
      module_balance.pending_withdrawal_amount - inscribe_withdraw.amount.clone();
    context.update_brc20_module_address_token_balance(
      &self.sender,
      &withdraw.module,
      &unique_lower_ticker,
      module_balance.clone(),
    )?;
    context.remove_brc20_module_inscribe_withdraw(self.old_satpoint)?;

    let available_balance = module_balance.available_balance.clone();
    if available_balance < inscribe_withdraw.amount.clone() {
      log::debug!(
        "brc20swap execute_transfer_withdraw amount invalid: {:?}, available_balance: {:?}",
        inscribe_withdraw.amount.clone(),
        available_balance.clone()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InsufficientWithdrawAmount(inscribe_withdraw.amount.clone(), available_balance),
      ));
    }

    let receiver = self
      .receiver
      .as_ref()
      .cloned()
      .unwrap_or_else(|| inscribe_withdraw.sender.clone());

    // load and update brc20 ticker balance
    let mut brc20_balance = match context.load_brc20_balance(&receiver, &unique_lower_ticker)? {
      Some(balance) => balance,
      None => BRC20Balance {
        ticker: unique_lower_ticker.clone(),
        total: 0,
        available: 0,
      },
    };
    let (amount_value, _) = inscribe_withdraw.amount.to_u128();
    brc20_balance.total += amount_value;
    brc20_balance.available += amount_value;
    context.update_brc20_balance(&receiver, &unique_lower_ticker, brc20_balance)?;

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

    Ok(BRC20Receipt {
      inscription_id: self.inscription_id,
      sequence_number: self.sequence_number,
      inscription_number: self.inscription_number,
      old_satpoint: self.old_satpoint,
      new_satpoint: self.new_satpoint,
      op_type: BRC20OpType::Withdraw,
      sender: self.sender.clone(),
      receiver: self
        .receiver
        .as_ref()
        .cloned()
        .unwrap_or_else(|| self.sender.clone()),
      result: Ok(BRC20Event::TransferWithdraw(event::TransferWithdrawEvent {
        ticker: unique_lower_ticker.clone(),
        amount: inscribe_withdraw.amount.clone().to_string(),
        module: withdraw.module.clone(),
      })),
    })
  }
}
