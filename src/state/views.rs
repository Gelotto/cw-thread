use cosmwasm_schema::cw_serde;

use super::models::{Attachment, NodeMetadata};

#[cw_serde]
pub struct NodeView {
    pub metadata: NodeMetadata,
    pub body: String,
    pub attachments: Vec<Attachment>,
}
