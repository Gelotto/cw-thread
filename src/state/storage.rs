use cosmwasm_std::Addr;
use cw_lib::models::Owner;
use cw_storage_plus::{Item, Map};

use super::models::{Attachment, Config, FlagMetadata, NodeMetadata, TableInfo};

pub const OWNER: Item<Owner> = Item::new("owner");
pub const CONFIG: Item<Config> = Item::new("config");
pub const TABLE: Item<TableInfo> = Item::new("table");

pub const NODE_ID_COUNTER: Item<u32> = Item::new("node_id_counter");
pub const NODE_ID_2_METADATA: Map<u32, NodeMetadata> = Map::new("node_id_2_metadata");
pub const NODE_ID_2_BODY: Map<u32, String> = Map::new("node_id_2_body");
pub const NODE_ID_2_ATTACHMENT: Map<(u32, u8), Attachment> = Map::new("node_id_2_attachment");
pub const NODE_ID_2_FLAG: Map<(u32, &Addr), FlagMetadata> = Map::new("node_id_2_flag");
pub const NODE_ID_ADDR_2_SENTIMENT: Map<(u32, &Addr), u8> = Map::new("node_id_addr_2_sentiment");

pub const NEG_REPLY_RELATIONSHIP: Map<(u32, u32, u32), bool> = Map::new("neg_reply_rel");
pub const POS_REPLY_RELATIONSHIP: Map<(u32, u32, u32), bool> = Map::new("pos_reply_rel");

pub const CHILD_RELATIONSHIP: Map<(u32, u32), bool> = Map::new("child_rel");

pub const CALLOUT_NODE_RELATIONSHIP: Map<(&String, u32), bool> = Map::new("callout_rel");
pub const NODE_CALLOUT_RELATIONSHIP: Map<(u32, &String), bool> = Map::new("node_callout_rel");

pub const HASHTAG_NODE_RELATIONSHIP: Map<(&String, u32), bool> = Map::new("hashtag_rel");
pub const NODE_HASHTAG_RELATIONSHIP: Map<(u32, &String), bool> = Map::new("node_hashtag_rel");
