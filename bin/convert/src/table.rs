use ord::define_table;
use redb::TableDefinition;

define_table! { INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER, i32, u32 }
define_table! { INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER2, i64, u32 }

define_table! { SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY, u32, InscriptionEntryValue }
define_table! { SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY2, u32, InscriptionEntryValue2 }

type InscriptionIdValue = (u128, u128, u32);
type InscriptionEntryValue = (
  u16,                // charms
  u64,                // fee
  u32,                // height
  InscriptionIdValue, // inscription id
  i32,                // inscription number
  Vec<u32>,           // parents
  Option<u64>,        // sat
  u32,                // sequence number
  u32,                // timestamp
);
type InscriptionEntryValue2 = (
  u16,                // charms
  u64,                // fee
  u32,                // height
  InscriptionIdValue, // inscription id
  i64,                // inscription number
  Vec<u32>,           // parents
  Option<u64>,        // sat
  u32,                // sequence number
  u32,                // timestamp
);
