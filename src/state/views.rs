use std::marker::PhantomData;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Order, Storage};
use cw_lib::models::TokenV2;
use cw_storage_plus::Bound;

use crate::{error::ContractError, msg::Sentiment};

use super::{
    models::{NodeMetadata, Section},
    storage::{
        CONFIG_TIP_TOKEN_ALLOWLIST, IX_ADDR_SAVED_ID, IX_NODE_MENTION, IX_NODE_TAG, NODE_ID_2_BODY,
        NODE_ID_2_MENTIONS, NODE_ID_2_METADATA, NODE_ID_2_SECTION, NODE_ID_2_TAGS, NODE_ID_2_TITLE,
        NODE_ID_ADDR_2_SENTIMENT,
    },
};

#[cw_serde]
pub struct NodeAccountView {
    pub sentiment: Option<Sentiment>,
    pub saved: bool,
}

#[cw_serde]
pub struct NodeView {
    pub metadata: NodeMetadata,
    pub title: Option<String>,
    pub body: String,
    pub sections: Vec<Section>,
    pub account: Option<NodeAccountView>,
    pub tags: Vec<String>,
    pub mentions: Vec<String>,
}

impl NodeView {
    pub fn load(
        store: &dyn Storage,
        id: u32,
        account_addr: &Option<Addr>,
    ) -> Result<NodeView, ContractError> {
        let metadata = NODE_ID_2_METADATA.load(store, id)?;
        let body = NODE_ID_2_BODY.load(store, id)?;
        let mentions = NODE_ID_2_MENTIONS.load(store, id)?;
        let tags = NODE_ID_2_TAGS.load(store, id)?;
        let title = NODE_ID_2_TITLE.may_load(store, id)?;
        let sections = NODE_ID_2_SECTION
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
            .collect::<Vec<Section>>();

        let account = match account_addr {
            None => None,
            Some(addr) => Some(NodeAccountView {
                saved: IX_ADDR_SAVED_ID.has(store, (addr, id)),
                sentiment: if let Some(sent_u8) =
                    NODE_ID_ADDR_2_SENTIMENT.may_load(store, (id, addr))?
                {
                    Sentiment::from_u8(sent_u8)
                } else {
                    None
                },
            }),
        };

        Ok(Self {
            metadata,
            title,
            body,
            mentions,
            tags,
            sections,
            account,
        })
    }
}

#[cw_serde]
pub struct ConfigView {
    pub tip_tokens: Vec<TokenV2>,
}

impl ConfigView {
    pub fn load(store: &dyn Storage) -> Result<Self, ContractError> {
        Ok(Self {
            tip_tokens: CONFIG_TIP_TOKEN_ALLOWLIST.load(store)?,
        })
    }
}

pub fn load_tags(
    store: &dyn Storage,
    node_id: u32,
) -> Result<Vec<String>, ContractError> {
    Ok(IX_NODE_TAG
        .prefix(node_id)
        .keys(store, None, None, Order::Ascending)
        .map(|k| k.unwrap())
        .collect())
}

pub fn load_mentions(
    store: &dyn Storage,
    node_id: u32,
) -> Result<Vec<String>, ContractError> {
    Ok(IX_NODE_MENTION
        .prefix(node_id)
        .keys(store, None, None, Order::Ascending)
        .map(|k| k.unwrap())
        .collect())
}
