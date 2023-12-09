use cosmwasm_schema::cw_serde;

use crate::state::models::{Attachment, Config};

#[cw_serde]
pub struct InstantiateMsg {
    pub body: String,
    pub name: Option<String>,
    pub attachments: Option<Vec<Attachment>>,
}

#[cw_serde]
pub struct NodeReplyMsg {
    pub body: String,
    pub reply_to_id: u32,
    pub attachments: Option<Vec<Attachment>>,
}

#[cw_serde]
pub struct NodeVoteMsg {
    pub node_id: u32,
    pub is_up_vote: bool,
}

#[cw_serde]
pub enum ExecuteMsg {
    SetConfig(Config),
    Reply(NodeReplyMsg),
    Vote(NodeVoteMsg),
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct ConfigResponse(pub Config);
