use std::collections::HashSet;

use cosmwasm_std::{Order, Storage};

use crate::{
    error::ContractError,
    state::{
        models::NodeMetadata,
        storage::{
            IX_MENTION_NODE, IX_NODE_MENTION, IX_NODE_TAG, IX_TAG_NODE, NODE_ID_2_MENTIONS,
            NODE_ID_2_METADATA, NODE_ID_2_TAGS, NODE_ID_COUNTER,
        },
    },
};

pub fn load_node_metadata(
    store: &dyn Storage,
    id: u32,
    strict: bool,
) -> Result<Option<NodeMetadata>, ContractError> {
    let maybe_metadata = NODE_ID_2_METADATA.may_load(store, id)?;
    if strict && maybe_metadata.is_none() {
        return Err(ContractError::NodeNotFound { node_id: id });
    }
    Ok(maybe_metadata)
}

pub fn next_node_id(store: &mut dyn Storage) -> Result<u32, ContractError> {
    Ok(NODE_ID_COUNTER.update(store, |n| -> Result<_, ContractError> { Ok(n + 1) })?)
}

pub fn process_tags_and_mentions(
    store: &mut dyn Storage,
    node_id: u32,
    maybe_tags: Option<Vec<String>>,
    maybe_mentions: Option<Vec<String>>,
    is_editing: bool,
) -> Result<(HashSet<String>, HashSet<String>), ContractError> {
    let mut tags: HashSet<String> = HashSet::with_capacity(2);
    let mut mentions: HashSet<String> = HashSet::with_capacity(2);

    NODE_ID_2_TAGS.save(store, node_id, &maybe_tags.clone().unwrap_or_default())?;
    NODE_ID_2_MENTIONS.save(store, node_id, &maybe_mentions.clone().unwrap_or_default())?;

    for token in maybe_tags.unwrap_or_default().iter() {
        let tag = token.to_lowercase();
        // TODO: validate tag
        if !tags.contains(&tag) {
            let tag = tag.to_owned();
            IX_TAG_NODE.save(store, (&tag, node_id), &true)?;
            IX_NODE_TAG.save(store, (node_id, &tag), &true)?;
            tags.insert(tag);
        }
    }
    for token in maybe_mentions.unwrap_or_default().iter() {
        // TODO: validate mention
        if let Some(mention) = token.strip_prefix("@") {
            let mention = mention.to_lowercase();
            if !mentions.contains(&mention) {
                let mention = mention.to_owned();
                IX_TAG_NODE.save(store, (&mention, node_id), &true)?;
                IX_NODE_TAG.save(store, (node_id, &mention), &true)?;
                mentions.insert(mention);
            }
        }
    }

    // We don't enter this block on creation, only update:
    if is_editing {
        // Remove old tags and mentions
        for tag in IX_NODE_TAG
            .prefix(node_id)
            .keys(store, None, None, Order::Ascending)
            .filter_map(|r| {
                let tag = r.unwrap();
                if !tags.contains(&tag) {
                    Some(tag)
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
        {
            IX_NODE_TAG.remove(store, (node_id, &tag));
            IX_TAG_NODE.remove(store, (&tag, node_id));
        }
        for mention in IX_NODE_MENTION
            .prefix(node_id)
            .keys(store, None, None, Order::Ascending)
            .filter_map(|r| {
                let mention = r.unwrap();
                if !mentions.contains(&mention) {
                    Some(mention)
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
        {
            IX_NODE_MENTION.remove(store, (node_id, &mention));
            IX_MENTION_NODE.remove(store, (&mention, node_id));
        }
    }

    Ok((tags, mentions))
}
