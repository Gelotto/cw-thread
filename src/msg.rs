use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_lib::models::Owner;
use cw_table::lifecycle::LifecycleExecuteMsg;

use crate::state::{
    models::{Attachment, Config, TableInfo, DOWN, UP},
    views::NodeView,
};

#[cw_serde]
pub enum Sentiment {
    Up,
    Down,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub body: Option<String>,
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub mentions: Option<Vec<String>>,
    pub attachments: Option<Vec<Attachment>>,
    pub owner: Option<Owner>,
}

#[cw_serde]
pub struct NodeReplyMsg {
    pub body: String,
    pub tags: Option<Vec<String>>,
    pub mentions: Option<Vec<String>>,
    pub parent_id: u32,
    pub attachments: Option<Vec<Attachment>>,
}

#[cw_serde]
pub struct NodeEditMsg {
    pub id: u32,
    pub title: Option<String>,
    pub body: Option<String>,
    pub attachments: Option<Vec<Attachment>>,
    pub tags: Option<Vec<String>>,
    pub mentions: Option<Vec<String>>,
}

#[cw_serde]
pub struct NodeVoteMsg {
    pub id: u32,
    pub sentiment: Sentiment,
}

impl Sentiment {
    pub fn from_u8(u8_sentiment: u8) -> Self {
        if u8_sentiment == UP {
            Self::Up
        } else {
            Self::Down
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Sentiment::Up => UP,
            Sentiment::Down => DOWN,
        }
    }
}

#[cw_serde]
pub enum ExecuteMsg {
    Lifecycle(LifecycleExecuteMsg),
    SetConfig(Config),
    Reply(NodeReplyMsg),
    Vote(NodeVoteMsg),
    VoteMany(Vec<NodeVoteMsg>),
    Edit(NodeEditMsg),
    Delete { id: u32 },
    Flag { id: u32, reason: Option<String> },
    Unflag { id: u32 },
}

#[cw_serde]
pub enum NodesQueryMsg {
    ByIds {
        ids: Vec<u32>,
        sender: Option<Addr>,
    },
    InReplyTo {
        id: u32,
        cursor: Option<(u32, i32, u32)>,
        sender: Option<Addr>,
        limit: Option<u8>,
    },
    AncestorsOf {
        id: u32,
        levels: Option<u8>,
        sender: Option<Addr>,
    },
    WithTag {
        tag: String,
        cursor: Option<u32>,
        sender: Option<Addr>,
    },
    WithMention {
        mention: String,
        cursor: Option<u32>,
        sender: Option<Addr>,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Thread { sender: Option<Addr> },
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
pub struct NodeViewRepliesPaginationResponse {
    pub nodes: Vec<NodeView>,
    pub cursor: Option<(u32, i32, u32)>,
}

#[cw_serde]
pub struct NodeViewByTagPaginationResponse {
    pub nodes: Vec<NodeView>,
    pub cursor: Option<u32>,
}
