use std::collections::HashSet;

use cosmwasm_std::{Order, Storage};

use crate::{
    error::ContractError,
    state::{
        models::NodeMetadata,
        storage::{
            MENTION_NODE_RELATIONSHIP, NODE_ID_2_METADATA, NODE_ID_COUNTER,
            NODE_MENTION_RELATIONSHIP, NODE_TAG_RELATIONSHIP, TAG_NODE_RELATIONSHIP,
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
    Ok(
        NODE_ID_COUNTER.update(store, |n| -> Result<_, ContractError> {
            if n == 1 {
                return Err(ContractError::NotAuthorized {
                    reason: "No more nodes allowed in thread".to_owned(),
                });
            }
            Ok(n - 1)
        })?,
    )
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

    for token in maybe_tags.unwrap_or_default().iter() {
        let tag = token.to_lowercase();
        // TODO: validate tag
        if !tags.contains(&tag) {
            let tag = tag.to_owned();
            TAG_NODE_RELATIONSHIP.save(store, (&tag, node_id), &true)?;
            NODE_TAG_RELATIONSHIP.save(store, (node_id, &tag), &true)?;
            tags.insert(tag);
        }
    }
    for token in maybe_mentions.unwrap_or_default().iter() {
        // TODO: validate mention
        if let Some(mention) = token.strip_prefix("@") {
            let mention = mention.to_lowercase();
            if !mentions.contains(&mention) {
                let mention = mention.to_owned();
                TAG_NODE_RELATIONSHIP.save(store, (&mention, node_id), &true)?;
                NODE_TAG_RELATIONSHIP.save(store, (node_id, &mention), &true)?;
                mentions.insert(mention);
            }
        }
    }

    if is_editing {
        // Remove old tags and mentions
        for tag in NODE_TAG_RELATIONSHIP
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
            NODE_TAG_RELATIONSHIP.remove(store, (node_id, &tag));
            TAG_NODE_RELATIONSHIP.remove(store, (&tag, node_id));
        }
        for mention in NODE_MENTION_RELATIONSHIP
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
            NODE_MENTION_RELATIONSHIP.remove(store, (node_id, &mention));
            MENTION_NODE_RELATIONSHIP.remove(store, (&mention, node_id));
        }
    }

    Ok((tags, mentions))
}
