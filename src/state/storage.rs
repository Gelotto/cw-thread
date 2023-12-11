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
pub const CHILD_RELATIONSHIP: Map<(u32, u32), bool> = Map::new("child_relationship");
pub const NEG_REPLY_RELATIONSHIP: Map<(u32, u32, u32), bool> =
    Map::new("negative_reply_relationship");
pub const POS_REPLY_RELATIONSHIP: Map<(u32, u32, u32), bool> =
    Map::new("positive_reply_relationship");
