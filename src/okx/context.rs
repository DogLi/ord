use super::{
  brc20::{
    entry::{
      BRC20Balance, BRC20BalanceValue, BRC20ModuleAddressLPTokenBalanceKey,
      BRC20ModuleAddressLPTokenBalanceKeyValue, BRC20ModuleAddressTokenBalanceKey,
      BRC20ModuleAddressTokenBalanceKeyValue, BRC20ModuleInfo, BRC20ModuleInfoValue,
      BRC20ModuleLPTokenBalance, BRC20ModuleLPTokenBalanceValue, BRC20ModulePoolPair,
      BRC20ModuleSwapPoolPairBalance, BRC20ModuleSwapPoolPairBalanceKey,
      BRC20ModuleSwapPoolPairBalanceKeyValue, BRC20ModuleSwapPoolPairBalanceValue,
      BRC20ModuleTokenBalance, BRC20ModuleTokenBalanceValue, BRC20Receipt, BRC20ReceiptsValue,
      BRC20TickerInfo, BRC20TickerInfoValue, BRC20TransferAsset, BRC20TransferAssetValue,
      InscribeWithdraw, InscribeWithdrawValue,
    },
    operation::{commit::CommitValue, Commit},
    BRC20Ticker,
  },
  composite_key::AddressTickerKey,
  entry::{
    AddressTickerKeyValue, CollectionType, DynamicEntry, InscriptionReceipt,
    InscriptionReceiptsValue,
  },
  *,
};
use crate::index::entry::{Entry, SatPointValue, TxidValue};
use redb::{MultimapTable, ReadableTable, Table};

pub(crate) struct TableContext<'a, 'txn> {
  inscription_receipts: &'a mut Table<'txn, &'static TxidValue, &'static InscriptionReceiptsValue>,
  // BRC20 tables
  brc20_balances: &'a mut Table<'txn, &'static AddressTickerKeyValue, &'static BRC20BalanceValue>,
  brc20_ticker_info:
    &'a mut Table<'txn, &'static AddressTickerKeyValue, &'static BRC20TickerInfoValue>,
  brc20_receipts: &'a mut Table<'txn, &'static TxidValue, &'static BRC20ReceiptsValue>,
  brc20_satpoint_to_transfer_assets:
    &'a mut Table<'txn, &'static SatPointValue, &'static BRC20TransferAssetValue>,
  brc20_address_ticker_to_transfer_assets:
    &'a mut MultimapTable<'txn, &'static AddressTickerKeyValue, &'static SatPointValue>,
  sequence_number_to_collection_type: &'a mut Table<'txn, u32, u16>,
  bitmap_block_height_to_sequence_number: &'a mut Table<'txn, u32, u32>,
  btc_domain_to_sequence_number: &'a mut Table<'txn, &'static str, u32>,
  brc20_module_info: &'a mut Table<'txn, &'static str, &'static BRC20ModuleInfoValue>,
  brc20_module_address_ticker_balances: &'a mut Table<
    'txn,
    &'static BRC20ModuleAddressTokenBalanceKeyValue,
    &'static BRC20ModuleTokenBalanceValue,
  >,
  brc20_module_swap_poolpair_balances: &'a mut Table<
    'txn,
    &'static BRC20ModuleSwapPoolPairBalanceKeyValue,
    &'static BRC20ModuleSwapPoolPairBalanceValue,
  >,
  brc20_swap_commit_info: &'a mut Table<'txn, &'static str, &'static CommitValue>,
  brc20_module_inscribe_withdraws:
    &'a mut Table<'txn, &'static SatPointValue, &'static InscribeWithdrawValue>,
  brc20_module_address_lp_token_balance: &'a mut Table<
    'txn,
    &'static BRC20ModuleAddressLPTokenBalanceKeyValue,
    &'static BRC20ModuleLPTokenBalanceValue,
  >,
}

impl<'a, 'txn> TableContext<'a, 'txn> {
  pub fn new(
    inscription_receipts: &'a mut Table<
      'txn,
      &'static TxidValue,
      &'static InscriptionReceiptsValue,
    >,
    brc20_balances: &'a mut Table<'txn, &'static AddressTickerKeyValue, &'static BRC20BalanceValue>,
    brc20_ticker_info: &'a mut Table<
      'txn,
      &'static AddressTickerKeyValue,
      &'static BRC20TickerInfoValue,
    >,
    brc20_receipts: &'a mut Table<'txn, &'static TxidValue, &'static BRC20ReceiptsValue>,
    brc20_satpoint_to_transfer_assets: &'a mut Table<
      'txn,
      &'static SatPointValue,
      &'static BRC20TransferAssetValue,
    >,
    brc20_address_ticker_to_transfer_assets: &'a mut MultimapTable<
      'txn,
      &'static AddressTickerKeyValue,
      &'static SatPointValue,
    >,
    sequence_number_to_collection_type: &'a mut Table<'txn, u32, u16>,
    bitmap_block_height_to_sequence_number: &'a mut Table<'txn, u32, u32>,
    btc_domain_to_sequence_number: &'a mut Table<'txn, &'static str, u32>,
    brc20_module_info: &'a mut Table<'txn, &'static str, &'static BRC20ModuleInfoValue>,
    brc20_module_address_ticker_balances: &'a mut Table<
      'txn,
      &'static BRC20ModuleAddressTokenBalanceKeyValue,
      &'static BRC20ModuleTokenBalanceValue,
    >,
    brc20_module_swap_poolpair_balances: &'a mut Table<
      'txn,
      &'static BRC20ModuleSwapPoolPairBalanceKeyValue,
      &'static BRC20ModuleSwapPoolPairBalanceValue,
    >,
    brc20_swap_commit_info: &'a mut Table<'txn, &'static str, &'static CommitValue>,
    brc20_module_inscribe_withdraws: &'a mut Table<
      'txn,
      &'static SatPointValue,
      &'static InscribeWithdrawValue,
    >,
    brc20_module_address_lp_token_balance: &'a mut Table<
      'txn,
      &'static BRC20ModuleAddressLPTokenBalanceKeyValue,
      &'static BRC20ModuleLPTokenBalanceValue,
    >,
  ) -> Self {
    Self {
      inscription_receipts,
      brc20_balances,
      brc20_ticker_info,
      brc20_receipts,
      brc20_satpoint_to_transfer_assets,
      brc20_address_ticker_to_transfer_assets,
      sequence_number_to_collection_type,
      bitmap_block_height_to_sequence_number,
      btc_domain_to_sequence_number,
      brc20_module_info,
      brc20_module_address_ticker_balances,
      brc20_module_swap_poolpair_balances,
      brc20_swap_commit_info,
      brc20_module_inscribe_withdraws,
      brc20_module_address_lp_token_balance,
    }
  }

  pub fn load_brc20_ticker_info(
    &mut self,
    ticker: &BRC20Ticker,
  ) -> Result<Option<BRC20TickerInfo>, redb::StorageError> {
    Ok(
      self
        .brc20_ticker_info
        .get(ticker.to_lowercase().store().as_ref())?
        .map(|v| DynamicEntry::load(v.value())),
    )
  }

  pub fn update_brc20_ticker_info(
    &mut self,
    ticker: &BRC20Ticker,
    info: BRC20TickerInfo,
  ) -> Result<(), redb::StorageError> {
    self.brc20_ticker_info.insert(
      ticker.to_lowercase().store().as_ref(),
      info.store().as_ref(),
    )?;
    Ok(())
  }

  pub fn load_brc20_balance(
    &mut self,
    address: &UtxoAddress,
    ticker: &BRC20Ticker,
  ) -> Result<Option<BRC20Balance>, redb::StorageError> {
    Ok(
      self
        .brc20_balances
        .get(
          AddressTickerKey {
            primary: address.clone(),
            secondary: ticker.to_lowercase().clone(),
          }
          .store()
          .as_ref(),
        )?
        .map(|v| DynamicEntry::load(v.value())),
    )
  }

  pub fn update_brc20_balance(
    &mut self,
    address: &UtxoAddress,
    ticker: &BRC20Ticker,
    balance: BRC20Balance,
  ) -> Result<(), redb::StorageError> {
    self.brc20_balances.insert(
      AddressTickerKey {
        primary: address.clone(),
        secondary: ticker.to_lowercase().clone(),
      }
      .store()
      .as_ref(),
      balance.store().as_ref(),
    )?;
    Ok(())
  }

  pub fn load_brc20_transferring_asset(
    &mut self,
    satpoint: SatPoint,
  ) -> std::result::Result<Option<BRC20TransferAsset>, redb::StorageError> {
    Ok(
      self
        .brc20_satpoint_to_transfer_assets
        .get(&satpoint.store())?
        .map(|v| DynamicEntry::load(v.value())),
    )
  }

  pub fn remove_brc20_transferring_asset(
    &mut self,
    satpoint: SatPoint,
  ) -> std::result::Result<(), redb::StorageError> {
    if let Some(asset) = self
      .brc20_satpoint_to_transfer_assets
      .remove(&satpoint.store())?
      .map(|v| BRC20TransferAsset::load(v.value()))
    {
      self.brc20_address_ticker_to_transfer_assets.remove(
        AddressTickerKey {
          primary: asset.owner,
          secondary: asset.ticker.to_lowercase(),
        }
        .store()
        .as_ref(),
        &satpoint.store(),
      )?;
    }
    Ok(())
  }

  pub fn insert_brc20_transferring_asset(
    &mut self,
    address: &UtxoAddress,
    ticker: &BRC20Ticker,
    satpoint: SatPoint,
    asset: BRC20TransferAsset,
  ) -> std::result::Result<(), redb::StorageError> {
    self
      .brc20_satpoint_to_transfer_assets
      .insert(&satpoint.store(), asset.store().as_ref())?;
    self.brc20_address_ticker_to_transfer_assets.insert(
      AddressTickerKey {
        primary: address.clone(),
        secondary: ticker.to_lowercase().clone(),
      }
      .store()
      .as_ref(),
      &satpoint.store(),
    )?;
    Ok(())
  }

  pub fn insert_brc20_tx_receipts(
    &mut self,
    txid: &Txid,
    receipts: Vec<BRC20Receipt>,
  ) -> std::result::Result<(), redb::StorageError> {
    self
      .brc20_receipts
      .insert(&txid.store(), receipts.store().as_ref())?;
    Ok(())
  }

  pub fn insert_inscription_tx_receipts(
    &mut self,
    txid: &Txid,
    receipts: Vec<InscriptionReceipt>,
  ) -> std::result::Result<(), redb::StorageError> {
    self
      .inscription_receipts
      .insert(&txid.store(), receipts.store().as_ref())?;
    Ok(())
  }

  pub fn load_bitmap_block_height_to_sequence_number(
    &mut self,
    height: u32,
  ) -> Result<Option<u32>, redb::StorageError> {
    Ok(
      self
        .bitmap_block_height_to_sequence_number
        .get(&height)?
        .map(|v| v.value()),
    )
  }

  pub fn load_btc_domain_to_sequence_number(
    &mut self,
    domain: &str,
  ) -> Result<Option<u32>, redb::StorageError> {
    Ok(
      self
        .btc_domain_to_sequence_number
        .get(domain)?
        .map(|v| v.value()),
    )
  }

  pub fn insert_bitmap_block_height_to_sequence_number(
    &mut self,
    height: u32,
    sequence_number: u32,
  ) -> Result<(), redb::StorageError> {
    self
      .bitmap_block_height_to_sequence_number
      .insert(height, sequence_number)?;
    Ok(())
  }

  pub fn insert_btc_domain_to_sequence_number(
    &mut self,
    domain: &str,
    sequence_number: u32,
  ) -> Result<(), redb::StorageError> {
    self
      .btc_domain_to_sequence_number
      .insert(domain, sequence_number)?;
    Ok(())
  }

  pub fn insert_sequence_number_to_collection_type(
    &mut self,
    sequence_number: u32,
    collection_type: CollectionType,
  ) -> Result<(), redb::StorageError> {
    self
      .sequence_number_to_collection_type
      .insert(sequence_number, u16::from(collection_type))?;
    Ok(())
  }

  pub fn is_module_already_deployed(
    &mut self,
    module_id: &str,
  ) -> Result<bool, redb::StorageError> {
    Ok(self.brc20_module_info.get(module_id)?.is_some())
  }

  pub fn insert_brc20_module_info(
    &mut self,
    module_id: &str,
    module_info: BRC20ModuleInfo,
  ) -> Result<(), redb::StorageError> {
    self
      .brc20_module_info
      .insert(module_id, module_info.store().as_ref())?;
    Ok(())
  }

  pub fn load_brc20_module_info(
    &mut self,
    module_id: &str,
  ) -> Result<Option<BRC20ModuleInfo>, redb::StorageError> {
    Ok(
      self
        .brc20_module_info
        .get(module_id)?
        .map(|v| DynamicEntry::load(v.value())),
    )
  }

  pub fn load_brc20_module_address_token_balance(
    &mut self,
    address: &UtxoAddress,
    module_id: &String,
    ticker: &BRC20Ticker,
  ) -> Result<Option<BRC20ModuleTokenBalance>, redb::StorageError> {
    Ok(
      self
        .brc20_module_address_ticker_balances
        .get(
          BRC20ModuleAddressTokenBalanceKey {
            address: address.clone(),
            module_id: module_id.to_string(),
            ticker: ticker.to_lowercase().clone(),
          }
          .store()
          .as_ref(),
        )?
        .map(|v| DynamicEntry::load(v.value())),
    )
  }

  pub fn update_brc20_module_address_token_balance(
    &mut self,
    address: &UtxoAddress,
    module_id: &String,
    ticker: &BRC20Ticker,
    balance: BRC20ModuleTokenBalance,
  ) -> Result<(), redb::StorageError> {
    self.brc20_module_address_ticker_balances.insert(
      BRC20ModuleAddressTokenBalanceKey {
        address: address.clone(),
        module_id: module_id.to_string(),
        ticker: ticker.to_lowercase().clone(),
      }
      .store()
      .as_ref(),
      balance.store().as_ref(),
    )?;
    Ok(())
  }

  pub fn load_brc20_module_swap_poolpair_balance(
    &mut self,
    module_id: &str,
    pool_pair: &BRC20ModulePoolPair,
  ) -> Result<Option<BRC20ModuleSwapPoolPairBalance>, redb::StorageError> {
    Ok(
      self
        .brc20_module_swap_poolpair_balances
        .get(
          BRC20ModuleSwapPoolPairBalanceKey {
            module_id: module_id.to_string(),
            pool_pair: pool_pair.clone(),
          }
          .store()
          .as_ref(),
        )?
        .map(|v| DynamicEntry::load(v.value())),
    )
  }

  pub fn update_brc20_module_swap_poolpair_balance(
    &mut self,
    module_id: &str,
    pool_pair: &BRC20ModulePoolPair,
    balance: BRC20ModuleSwapPoolPairBalance,
  ) -> Result<(), redb::StorageError> {
    self.brc20_module_swap_poolpair_balances.insert(
      BRC20ModuleSwapPoolPairBalanceKey {
        module_id: module_id.to_string(),
        pool_pair: pool_pair.clone(),
      }
      .store()
      .as_ref(),
      balance.store().as_ref(),
    )?;
    Ok(())
  }

  pub fn load_commit_info(
    &mut self,
    inscription_id: &str,
  ) -> Result<Option<Commit>, redb::StorageError> {
    Ok(
      self
        .brc20_swap_commit_info
        .get(inscription_id)?
        .map(|v| DynamicEntry::load(v.value())),
    )
  }
  pub fn insert_brc20_module_withdraw(
    &mut self,
    satpoint: SatPoint,
    withdraw: InscribeWithdraw,
  ) -> Result<(), redb::StorageError> {
    self
      .brc20_module_inscribe_withdraws
      .insert(&satpoint.store(), withdraw.store().as_ref())?;
    Ok(())
  }

  pub fn load_brc20_module_inscribe_withdraw(
    &mut self,
    satpoint: SatPoint,
  ) -> Result<Option<InscribeWithdraw>, redb::StorageError> {
    Ok(
      self
        .brc20_module_inscribe_withdraws
        .get(&satpoint.store())?
        .map(|v| DynamicEntry::load(v.value())),
    )
  }

  pub fn update_commit_info(
    &mut self,
    inscription_id: &str,
    commit: Commit,
  ) -> Result<(), redb::StorageError> {
    self
      .brc20_swap_commit_info
      .insert(inscription_id, commit.store().as_ref())?;
    Ok(())
  }

  pub fn delete_commit_info(&mut self, inscription_id: &str) -> Result<(), redb::StorageError> {
    self.brc20_swap_commit_info.remove(inscription_id)?;
    Ok(())
  }

  pub fn remove_brc20_module_inscribe_withdraw(
    &mut self,
    satpoint: SatPoint,
  ) -> Result<(), redb::StorageError> {
    self
      .brc20_module_inscribe_withdraws
      .remove(&satpoint.store())?;
    Ok(())
  }

  pub fn load_brc20_module_address_lp_token_balance(
    &mut self,
    lp_key: &BRC20ModuleAddressLPTokenBalanceKey,
  ) -> Result<Option<BRC20ModuleLPTokenBalance>, redb::StorageError> {
    Ok(
      self
        .brc20_module_address_lp_token_balance
        .get(lp_key.store().as_ref())?
        .map(|v| DynamicEntry::load(v.value())),
    )
  }

  pub fn update_brc20_module_address_lp_token_balance(
    &mut self,
    lp_key: &BRC20ModuleAddressLPTokenBalanceKey,
    balance: BRC20ModuleLPTokenBalance,
  ) -> Result<(), redb::StorageError> {
    self
      .brc20_module_address_lp_token_balance
      .insert(lp_key.store().as_ref(), balance.store().as_ref())?;
    Ok(())
  }
}
