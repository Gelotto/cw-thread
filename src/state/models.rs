use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Storage, Timestamp};
use cw_table::client::Table;

use crate::error::ContractError;

use super::storage::TABLE;

pub const NIL: u8 = 0;
pub const DOWN: u8 = 1;
pub const UP: u8 = 2;

pub const ROOT_ID: u32 = 0;

#[cw_serde]
pub struct TableMetadata {
    pub address: Addr,
    pub id: String,
}

impl TableMetadata {
    pub fn load_client(
        store: &dyn Storage,
        contract_addr: &Addr,
    ) -> Result<Option<Table>, ContractError> {
        if let Some(table_info) = TABLE.may_load(store)? {
            Ok(Some(Table::new(&table_info.address, contract_addr)))
        } else {
            Ok(None)
        }
    }
}

#[cw_serde]
pub enum Section {
    Image {
        uri: String,
        caption: Option<String>,
    },
    Link {
        url: String,
        name: Option<String>,
    },
    Code {
        language: Option<String>,
        caption: Option<String>,
        text: String,
    },
    Text {
        title: Option<String>,
        body: Option<String>,
    },
}

#[cw_serde]
pub struct NodeMetadata {
    pub id: u32,
    pub parent_id: Option<u32>,
    pub created_at: Timestamp,
    pub updated_at: Option<Timestamp>,
    pub created_by: Addr,
    pub rank: i32,
    pub n_replies: u16,
    pub n_sections: u8,
    pub n_flags: u8,
    pub depth: u8,
}

#[cw_serde]
pub struct FlagMetadata {
    pub flagged_at: Timestamp,
    pub flagged_by: Addr,
    pub reason: Option<String>,
}
