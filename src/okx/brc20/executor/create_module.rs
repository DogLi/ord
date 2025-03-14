use super::*;
use crate::okx::brc20::{
  brc20_decimal::Brc20Decimal,
  entry::BRC20ModuleInfo,
  event::{CreateModuleEvent, CreateModuleInitFields},
  verify::check_ticker_verify,
};

impl BRC20ExecutionMessage {
  pub(super) fn execute_create_module(
    &self,
    inscription_id: &InscriptionId,
    index: &Index,
    context: &mut TableContext,
  ) -> Result<BRC20Receipt, ExecutionError> {
    log::info!(
      "brc20swap execute_create_module: {:?}, inscription_id: {:?}",
      self.txid,
      self.inscription_id
    );

    let BRC20Operation::CreateModule(create_module) = &self.operation else {
      log::debug!(
        "brc20swap error execute_create_module, operation is not CreateModule inscription_id: {:?}",
        inscription_id
      );
      unreachable!()
    };

    if index.brc20_swap_source() != create_module.source {
      log::debug!(
        "brc20swap error create_module, module_source_not_match: {:?}, {:?}",
        index.brc20_swap_source(),
        create_module.source
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::ModuleSourceNotMatch(create_module.source.clone()),
      ));
    }

    if context.is_module_already_deployed(&inscription_id.to_string())? {
      log::debug!(
        "brc20swap error create_module, module_already_deployed: {:?}",
        inscription_id.to_string()
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::ModuleAlreadyExists(inscription_id.to_string()),
      ));
    }

    let swap_fee_rate = if create_module.init.swap_fee_rate.is_empty() {
      "0"
    } else {
      create_module.init.swap_fee_rate.as_str()
    };
    let swap_fee_rate_decimal =
      Brc20Decimal::from_str_with_scale(swap_fee_rate, 3).map_err(|_| {
        log::debug!(
          "brc20swap errorcreate_module, invalid swap_fee_rate: {}",
          swap_fee_rate
        );
        ExecutionError::ExecutionFailed(BRC20Error::InvalidSwapFeeRate(swap_fee_rate.to_string()))
      })?;
    if swap_fee_rate_decimal.sign() < 0 {
      log::debug!(
        "brc20swap error create_module, invalid swap_fee_rate_decimal: {}",
        swap_fee_rate_decimal
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InvalidSwapFeeRate(swap_fee_rate.to_string()),
      ));
    }

    let gas_tick = create_module.init.gas_tick.as_str();
    if gas_tick.is_empty() {
      log::debug!(
        "brc20swap error create_module, invalid gas_tick: {}",
        gas_tick
      );
      return Err(ExecutionError::ExecutionFailed(BRC20Error::InvalidGasTick(
        "empty gas tick".to_string(),
      )));
    }
    if check_ticker_verify(
      context,
      &BRC20Ticker::from_str(&gas_tick).unwrap(),
      &String::new(),
    )
    .is_err()
    {
      log::debug!(
        "brc20swap error create_module, invalid gas_tick: {}",
        gas_tick
      );
      return Err(ExecutionError::ExecutionFailed(BRC20Error::InvalidGasTick(
        gas_tick.to_string(),
      )));
    }

    if create_module.init.sequencer.is_empty() {
      log::debug!(
        "brc20swap error create_module, invalid sequencer: {}",
        create_module.init.sequencer
      );
      return Err(ExecutionError::ExecutionFailed(
        BRC20Error::InvalidSequencer("empty sequencer".to_string()),
      ));
    }
    let sequencer_pk_script =
      match UtxoAddress::get_script_by_str(&create_module.init.sequencer.clone(), &index.chain()) {
        Ok(pk_script) => pk_script.to_hex_string(),
        Err(e) => {
          log::debug!(
            "brc20swap error create_module, invalid sequencer: {}",
            e.to_string()
          );
          return Err(ExecutionError::ExecutionFailed(
            BRC20Error::InvalidSequencer(e.to_string()),
          ));
        }
      };

    if create_module.init.gas_to.is_empty() {
      log::debug!(
        "brc20swap error create_module, invalid gas_to: {}",
        create_module.init.gas_to
      );
      return Err(ExecutionError::ExecutionFailed(BRC20Error::InvalidGasTo(
        "empty gas to".to_string(),
      )));
    }
    let gas_to_pk_script =
      match UtxoAddress::get_script_by_str(&create_module.init.gas_to.clone(), &index.chain()) {
        Ok(pk_script) => pk_script.to_hex_string(),
        Err(e) => {
          log::debug!(
            "brc20swap error create_module, invalid gas_to: {}",
            e.to_string()
          );
          return Err(ExecutionError::ExecutionFailed(BRC20Error::InvalidGasTo(
            e.to_string(),
          )));
        }
      };

    if create_module.init.fee_to.is_empty() {
      log::debug!(
        "brc20swap error create_module, invalid fee_to: {}",
        create_module.init.fee_to
      );
      return Err(ExecutionError::ExecutionFailed(BRC20Error::InvalidFeeTo(
        "empty fee to".to_string(),
      )));
    }
    let fee_to_pk_script =
      match UtxoAddress::get_script_by_str(&create_module.init.fee_to.clone(), &index.chain()) {
        Ok(pk_script) => pk_script.to_hex_string(),
        Err(e) => {
          log::debug!(
            "brc20swap error create_module, invalid fee_to: {}",
            e.to_string()
          );
          return Err(ExecutionError::ExecutionFailed(BRC20Error::InvalidFeeTo(
            e.to_string(),
          )));
        }
      };

    let module_info = BRC20ModuleInfo {
      id: inscription_id.to_string(),
      name: create_module.name.clone(),
      deployer_pk_script: self.sender.to_script(&index.chain()).to_hex_string(),
      sequencer_pk_script,
      gas_to_pk_script,
      lp_fee_pk_script: fee_to_pk_script,
      fee_rate_swap: swap_fee_rate_decimal,
      gas_tick: gas_tick.to_string(),
      chain_commit_id: None,
      commit_id: None,
    };

    match context.insert_brc20_module_info(&inscription_id.to_string(), module_info.clone()) {
      Ok(_) => log::debug!(
        "brc20swap create_module, insert_brc20_module_info success: {:?}",
        module_info.clone()
      ),
      Err(e) => {
        return Err(ExecutionError::ExecutionFailed(BRC20Error::InvalidFeeTo(
          e.to_string(),
        )));
      }
    }

    log::debug!("brc20swap execute_create_module finished: {:?}", self);
    Ok(BRC20Receipt {
      inscription_id: self.inscription_id,
      sequence_number: self.sequence_number,
      inscription_number: self.inscription_number,
      old_satpoint: self.old_satpoint,
      new_satpoint: self.new_satpoint,
      op_type: BRC20OpType::CreateModule,
      sender: self.sender.clone(),
      receiver: self.receiver.clone().unwrap_or(self.sender.clone()),
      result: Ok(BRC20Event::CreateModule(CreateModuleEvent {
        name: create_module.name.clone(),
        source: create_module.source.clone(),
        init: CreateModuleInitFields {
          swap_fee_rate: create_module.init.swap_fee_rate.clone(),
          gas_tick: create_module.init.gas_tick.clone(),
          gas_to: create_module.init.gas_to.clone(),
          fee_to: create_module.init.fee_to.clone(),
          sequencer: create_module.init.sequencer.clone(),
        },
      })),
    })
  }
}
