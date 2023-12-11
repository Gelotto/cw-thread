use cosmwasm_schema::cw_serde;
use cw_lib::models::Owner;
use cw_table::lifecycle::LifecycleExecuteMsg;

use crate::state::{
    models::{Attachment, Config, TableInfo},
    views::NodeView,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub body: String,
    pub name: Option<String>,
    pub attachments: Option<Vec<Attachment>>,
    pub owner: Option<Owner>,
}

#[cw_serde]
pub struct NodeReplyMsg {
    pub body: String,
    pub reply_to_id: u32,
    pub attachments: Option<Vec<Attachment>>,
}

#[cw_serde]
pub struct NodeVoteMsg {
    pub id: u32,
    pub is_positive: bool,
}

#[cw_serde]
pub enum ExecuteMsg {
    Lifecycle(LifecycleExecuteMsg),
    SetConfig(Config),
    Reply(NodeReplyMsg),
    Vote(NodeVoteMsg),
    Flag { id: u32, reason: Option<String> },
    Unflag { id: u32 },
}

#[cw_serde]
pub enum NodesQueryMsg {
    ById(Vec<u32>),
    InReplyTo {
        id: u32,
        cursor: Option<(u8, u32, u32)>,
    },
    AncestorsOf {
        id: u32,
        levels: Option<u8>,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Thread {},
    Nodes(NodesQueryMsg),
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct ConfigResponse(pub Config);

#[cw_serde]
pub struct ThreadInfoResponse {
    pub table: Option<TableInfo>,
    pub config: Config,
    pub owner: Owner,
    pub root: NodeView,
}

#[cw_serde]
pub struct ReplyNodeViewPaginationResponse {
    pub replies: Vec<NodeView>,
    pub cursor: Option<(u8, u32, u32)>,
}
