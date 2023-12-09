use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp};

pub const NEGATIVE_SENTIMENT: u8 = 0;
pub const POSITIVE_SENTIMENT: u8 = 1;

#[cw_serde]
pub struct Config {}

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
    pub n_replies: u16,
    pub n_attachments: u8,
    pub n_votes: u32,
    pub sentiment: u8,
}
