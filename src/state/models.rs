use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp};

pub const NIL: u8 = 0;
pub const DOWN: u8 = 1;
pub const UP: u8 = 2;

/// Since rank can be negative or positive yet ints in CW must be unsigned, we
/// use ZERO_RANK to represent 0, and anything less than this value is
/// considered a negative rank. With this, we only need one map to store ranks,
/// instead of one for each sentiment.
pub const RANK_ZERO: u32 = u32::MAX >> 1;

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
