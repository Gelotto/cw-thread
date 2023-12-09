use cw_storage_plus::{Item, Map};

use super::models::{Attachment, Config, NodeMetadata};

pub const CONFIG: Item<Config> = Item::new("config");
pub const THREAD_ID: Item<String> = Item::new("thread_id");
pub const NODE_ID_COUNTER: Item<u32> = Item::new("node_id_counter");
pub const NODE_ID_2_METADATA: Map<u32, NodeMetadata> = Map::new("node_id_2_metadata");
pub const NODE_ID_2_BODY: Map<u32, String> = Map::new("node_id_2_body");
pub const NODE_ID_2_ATTACHMENT: Map<(u32, u8), Attachment> = Map::new("node_id_2_attachment");
pub const IX_RANKED_REPLIES: Map<(u32, (u8, u32, u32)), bool> = Map::new("ix_ranked_replies");
