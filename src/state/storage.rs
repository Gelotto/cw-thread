use cosmwasm_std::Addr;
use cw_lib::models::Owner;
use cw_storage_plus::{Item, Map};

use super::models::{Config, FlagMetadata, NodeMetadata, Section, TableMetadata};

pub const OWNER: Item<Owner> = Item::new("owner");
pub const CONFIG: Item<Config> = Item::new("config");
pub const TABLE: Item<TableMetadata> = Item::new("table");

pub const NODE_ID_COUNTER: Item<u32> = Item::new("node_id_counter");
pub const NODE_ID_2_METADATA: Map<u32, NodeMetadata> = Map::new("node_id_2_metadata");
pub const NODE_ID_2_TITLE: Map<u32, String> = Map::new("node_id_2_title");
pub const NODE_ID_2_BODY: Map<u32, String> = Map::new("node_id_2_body");
pub const NODE_ID_2_SECTION: Map<(u32, u8), Section> = Map::new("node_id_2_section");
pub const NODE_ID_2_FLAG: Map<(u32, &Addr), FlagMetadata> = Map::new("node_id_2_flag");
pub const NODE_ID_ADDR_2_SENTIMENT: Map<(u32, &Addr), u8> = Map::new("node_id_addr_2_sentiment");

pub const RANKED_CHILDREN: Map<(u32, i32, u32), bool> = Map::new("ranked_child");
pub const CHILD_RELATIONSHIP: Map<(u32, u32), bool> = Map::new("child_rel");

pub const MENTION_NODE_RELATIONSHIP: Map<(&String, u32), bool> = Map::new("mention_rel");
pub const NODE_MENTION_RELATIONSHIP: Map<(u32, &String), bool> = Map::new("node_mention_rel");

pub const TAG_NODE_RELATIONSHIP: Map<(&String, u32), bool> = Map::new("tag_rel");
pub const NODE_TAG_RELATIONSHIP: Map<(u32, &String), bool> = Map::new("node_tag_rel");
