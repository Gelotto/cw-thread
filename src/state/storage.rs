use cosmwasm_std::{Addr, Uint128};
use cw_lib::models::{Owner, TokenV2};
use cw_storage_plus::{Item, Map};

use super::models::{FlagMetadata, NodeMetadata, Section, TableMetadata};

pub const MAX_TIP_TOKEN_TYPES: usize = 10;

// Validation limits
pub const MAX_TITLE_LENGTH: usize = 200;
pub const MAX_BODY_LENGTH: usize = 50_000;
pub const MAX_TAGS: usize = 10;
pub const MAX_TAG_LENGTH: usize = 30;
pub const MAX_MENTIONS: usize = 20;
pub const MAX_SECTIONS: usize = 20;

pub const OWNER: Item<Owner> = Item::new("owner");
pub const CONFIG_TIP_TOKEN_ALLOWLIST: Item<Vec<TokenV2>> = Item::new("config_tip_token_allowlist");
pub const TABLE: Item<TableMetadata> = Item::new("table");
pub const ACTIVITY_SCORE: Item<u32> = Item::new("activity_score");
pub const TIP_TOKEN_LUTAB: Map<&String, bool> = Map::new("tip_token_lutab");

pub const N_TOTAL_REPLIES: Item<u32> = Item::new("n_total_replies");
pub const TOTAL_TIP_AMOUNTS: Map<&String, Uint128> = Map::new("total_tip_amounts");

pub const NODE_ID_COUNTER: Item<u32> = Item::new("node_id_counter");
pub const NODE_ID_2_METADATA: Map<u32, NodeMetadata> = Map::new("node_id_2_metadata");
pub const NODE_ID_2_TITLE: Map<u32, String> = Map::new("node_id_2_title");
pub const NODE_ID_2_TAGS: Map<u32, Vec<String>> = Map::new("node_id_2_tags");
pub const NODE_ID_2_MENTIONS: Map<u32, Vec<String>> = Map::new("node_id_2_mentions");
pub const NODE_ID_2_BODY: Map<u32, String> = Map::new("node_id_2_body");
pub const NODE_ID_2_SECTION: Map<(u32, u8), Section> = Map::new("node_id_2_section");
pub const NODE_ID_2_FLAG: Map<(u32, &Addr), FlagMetadata> = Map::new("node_id_2_flag");
pub const NODE_ID_ADDR_2_SENTIMENT: Map<(u32, &Addr), u8> = Map::new("node_id_addr_2_sentiment");

pub const IX_CHILD: Map<(u32, u32), bool> = Map::new("ix_child");
pub const IX_RANKED_CHILD: Map<(u32, i32, u32), bool> = Map::new("ix_ranked_child");
pub const IX_MENTION_NODE: Map<(&String, u32), bool> = Map::new("ix_mention");
pub const IX_NODE_MENTION: Map<(u32, &String), bool> = Map::new("ix_node_mention");
pub const IX_TAG_NODE: Map<(&String, u32), bool> = Map::new("ix_tag");
pub const IX_NODE_TAG: Map<(u32, &String), bool> = Map::new("ix_node_tag");
pub const IX_ADDR_SAVED_ID: Map<(&Addr, u32), bool> = Map::new("ix_addr_saved_id");
