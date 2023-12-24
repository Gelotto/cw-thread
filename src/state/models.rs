use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp};

pub const NEGATIVE: u8 = 0;
pub const POSITIVE: u8 = 1;
pub const NEUTRAL: u8 = 2;

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
}

#[cw_serde]
pub struct NodeMetadata {
    pub id: u32,
    pub reply_to_id: Option<u32>,
    pub created_at: Timestamp,
    pub updated_at: Option<Timestamp>,
    pub created_by: Addr,
    pub sentiment: u8,
    pub rank: u32,
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
