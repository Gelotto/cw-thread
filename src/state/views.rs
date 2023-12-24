use std::marker::PhantomData;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Order, Storage};
use cw_storage_plus::Bound;

use crate::{error::ContractError, msg::Sentiment};

use super::{
    models::{Attachment, NodeMetadata, NEUTRAL},
    storage::{
        NODE_ID_2_ATTACHMENT, NODE_ID_2_BODY, NODE_ID_2_METADATA, NODE_ID_2_TITLE,
        NODE_ID_ADDR_2_SENTIMENT,
    },
};

#[cw_serde]
pub struct NodeAccountView {
    pub sentiment: Sentiment,
}

#[cw_serde]
pub struct NodeView {
    pub metadata: NodeMetadata,
    pub title: Option<String>,
    pub body: String,
    pub attachments: Vec<Attachment>,
    pub account: Option<NodeAccountView>,
}

impl NodeView {
    pub fn load(
        store: &dyn Storage,
        id: u32,
        account_addr: &Option<Addr>,
    ) -> Result<NodeView, ContractError> {
        let metadata = NODE_ID_2_METADATA.load(store, id)?;

        let body = NODE_ID_2_BODY.load(store, id)?;
        let title = NODE_ID_2_TITLE.may_load(store, id)?;

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

        let account = match account_addr {
            None => None,
            Some(addr) => Some(NodeAccountView {
                sentiment: Sentiment::from_u8(
                    NODE_ID_ADDR_2_SENTIMENT
                        .may_load(store, (id, addr))?
                        .unwrap_or(NEUTRAL),
                ),
            }),
        };

        Ok(Self {
            metadata,
            title,
            body,
            attachments,
            account,
        })
    }
}
