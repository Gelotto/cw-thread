use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp};

pub const NIL: u8 = 0;
pub const DOWN: u8 = 1;
pub const UP: u8 = 2;

pub const ROOT_ID: u32 = 0;

#[cw_serde]
pub struct Config {
    // TODO: make readonly if is_archived
    pub is_archived: bool,
}

#[cw_serde]
pub struct TableInfo {
    pub address: Addr,
    pub id: String,
}

#[cw_serde]
pub enum Attachment {
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
    pub n_attachments: u8,
    pub n_flags: u8,
}

#[cw_serde]
pub struct FlagMetadata {
    pub flagged_at: Timestamp,
    pub flagged_by: Addr,
    pub reason: Option<String>,
}
