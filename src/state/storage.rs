use cosmwasm_std::Addr;
use cw_lib::models::Owner;
use cw_storage_plus::{Item, Map};

use super::models::{Attachment, Config, FlagMetadata, NodeMetadata, TableInfo};

pub const OWNER: Item<Owner> = Item::new("owner");
pub const CONFIG: Item<Config> = Item::new("config");
pub const TABLE: Item<TableInfo> = Item::new("table");

pub const NODE_ID_COUNTER: Item<u32> = Item::new("node_id_counter");
pub const NODE_ID_2_METADATA: Map<u32, NodeMetadata> = Map::new("node_id_2_metadata");
pub const NODE_ID_2_TITLE: Map<u32, String> = Map::new("node_id_2_title");
pub const NODE_ID_2_BODY: Map<u32, String> = Map::new("node_id_2_body");
pub const NODE_ID_2_ATTACHMENT: Map<(u32, u8), Attachment> = Map::new("node_id_2_attachment");
pub const NODE_ID_2_FLAG: Map<(u32, &Addr), FlagMetadata> = Map::new("node_id_2_flag");

pub const RANK_RELATIONSHIP: Map<(u32, u32, u32), bool> = Map::new("rank_rel");

// XXX: Deprecated
pub const NODE_ID_ADDR_2_SENTIMENT: Map<(u32, &Addr), u8> = Map::new("node_id_addr_2_sentiment");
// XXX: Deprecated
pub const UP_REPLY_RELATIONSHIP: Map<(u32, u32, u32), bool> = Map::new("up_reply_rel");
// XXX: Deprecated
pub const DOWN_REPLY_RELATIONSHIP: Map<(u32, u32, u32), bool> = Map::new("down_reply_rel");

pub const CHILD_RELATIONSHIP: Map<(u32, u32), bool> = Map::new("child_rel");

pub const MENTION_NODE_RELATIONSHIP: Map<(&String, u32), bool> = Map::new("MENTION_rel");
pub const NODE_MENTION_RELATIONSHIP: Map<(u32, &String), bool> = Map::new("NODE_MENTION_rel");

pub const TAG_NODE_RELATIONSHIP: Map<(&String, u32), bool> = Map::new("tag_rel");
pub const NODE_TAG_RELATIONSHIP: Map<(u32, &String), bool> = Map::new("node_tag_rel");
