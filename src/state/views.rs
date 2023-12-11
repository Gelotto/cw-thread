use std::marker::PhantomData;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Order, Storage};
use cw_storage_plus::Bound;

use crate::error::ContractError;

use super::{
    models::{Attachment, NodeMetadata},
    storage::{NODE_ID_2_ATTACHMENT, NODE_ID_2_BODY, NODE_ID_2_METADATA},
};

#[cw_serde]
pub struct NodeView {
    pub metadata: NodeMetadata,
    pub body: String,
    pub attachments: Vec<Attachment>,
}

impl NodeView {
    pub fn load(
        store: &dyn Storage,
        id: u32,
    ) -> Result<NodeView, ContractError> {
        let metadata = NODE_ID_2_METADATA.load(store, id)?;
        let body = NODE_ID_2_BODY.load(store, id)?;
        let attachments = NODE_ID_2_ATTACHMENT
            .range(
                store,
                Some(Bound::Inclusive(((id, u8::MIN), PhantomData))),
                None,
                Order::Ascending,
            )
            .map(|r| {
                let (_k, v) = r.unwrap();
                v
            })
            .collect::<Vec<Attachment>>();

        Ok(Self {
            metadata,
            body,
            attachments,
        })
    }
}
