use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_lib::models::{Owner, TokenAmountV2, TokenV2};
use cw_table::lifecycle::LifecycleExecuteMsg;

use crate::state::{
    models::{Section, TableMetadata, DOWN, UP},
    views::{ConfigView, NodeView},
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
    pub sections: Option<Vec<Section>>,
    pub owner: Option<Owner>,
    pub config: ConfigView,
}

#[cw_serde]
pub struct NodeReplyMsg {
    pub body: String,
    pub tags: Option<Vec<String>>,
    pub mentions: Option<Vec<String>>,
    pub parent_id: u32,
    pub sections: Option<Vec<Section>>,
}

#[cw_serde]
pub struct NodeEditMsg {
    pub id: u32,
    pub title: Option<String>,
    pub body: Option<String>,
    pub sections: Option<Vec<Section>>,
    pub tags: Option<Vec<String>>,
    pub mentions: Option<Vec<String>>,
}

#[cw_serde]
pub struct NodeVoteMsg {
    pub id: u32,
    pub sentiment: Sentiment,
}

impl Sentiment {
    pub fn from_u8(u8_sentiment: u8) -> Option<Self> {
        if u8_sentiment == UP {
            Some(Self::Up)
        } else if u8_sentiment == DOWN {
            Some(Self::Down)
        } else {
            None
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
pub struct ConfigUpdateMsg {
    pub tip_tokens: Option<Vec<TokenV2>>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Lifecycle(LifecycleExecuteMsg),
    SetConfig(ConfigUpdateMsg),
    Reply(NodeReplyMsg),
    Save(Vec<u32>),
    Unsave(Vec<u32>),
    Vote(NodeVoteMsg),
    VoteMany(Vec<NodeVoteMsg>),
    Edit(NodeEditMsg),
    Delete { id: u32 },
    Flag { id: u32, reason: Option<String> },
    Unflag { id: u32 },
    Tip(TokenAmountV2),
}

#[cw_serde]
pub enum NodesQueryMsg {
    ByIds {
        ids: Vec<u32>,
        sender: Option<Addr>,
    },
    Children {
        id: u32,
        cursor: Option<(u32, i32, u32)>,
        sender: Option<Addr>,
        limit: Option<u8>,
    },
    Ancestors {
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
pub struct ConfigResponse(pub ConfigView);

#[cw_serde]
pub struct ThreadInfoResponse {
    pub table: Option<TableMetadata>,
    pub config: ConfigView,
    pub owner: Owner,
    pub root: NodeView,
    pub n_total_replies: u32,
    pub tips: Vec<TokenAmountV2>,
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
