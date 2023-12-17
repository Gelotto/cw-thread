use std::collections::HashSet;

use cosmwasm_std::{Order, Storage};

use crate::{
    error::ContractError,
    state::{
        models::NodeMetadata,
        storage::{
            CALLOUT_NODE_RELATIONSHIP, HASHTAG_NODE_RELATIONSHIP, NODE_CALLOUT_RELATIONSHIP,
            NODE_HASHTAG_RELATIONSHIP, NODE_ID_2_METADATA, NODE_ID_COUNTER,
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

pub fn process_hashtags_and_callouts(
    store: &mut dyn Storage,
    node_id: u32,
    body: &String,
    is_editing: bool,
) -> Result<(HashSet<String>, HashSet<String>), ContractError> {
    let mut hashtags: HashSet<String> = HashSet::with_capacity(2);
    let mut callouts: HashSet<String> = HashSet::with_capacity(2);

    for token in body.split_whitespace() {
        if token.starts_with("#") {
            if let Some(tag) = token.strip_prefix("#") {
                let tag = tag.to_lowercase();
                if !hashtags.contains(&tag) {
                    let tag = tag.to_owned();
                    HASHTAG_NODE_RELATIONSHIP.save(store, (&tag, node_id), &true)?;
                    NODE_HASHTAG_RELATIONSHIP.save(store, (node_id, &tag), &true)?;
                    hashtags.insert(tag);
                }
            }
        } else if token.starts_with("@") {
            if let Some(callout) = token.strip_prefix("@") {
                let callout = callout.to_lowercase();
                if !callouts.contains(&callout) {
                    let callout = callout.to_owned();
                    HASHTAG_NODE_RELATIONSHIP.save(store, (&callout, node_id), &true)?;
                    NODE_HASHTAG_RELATIONSHIP.save(store, (node_id, &callout), &true)?;
                    callouts.insert(callout);
                }
            }
        }
    }

    if is_editing {
        // Remove old tags and callouts
        for tag in NODE_HASHTAG_RELATIONSHIP
            .prefix(node_id)
            .keys(store, None, None, Order::Ascending)
            .filter_map(|r| {
                let tag = r.unwrap();
                if !hashtags.contains(&tag) {
                    Some(tag)
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
        {
            NODE_HASHTAG_RELATIONSHIP.remove(store, (node_id, &tag));
            HASHTAG_NODE_RELATIONSHIP.remove(store, (&tag, node_id));
        }
        for callout in NODE_CALLOUT_RELATIONSHIP
            .prefix(node_id)
            .keys(store, None, None, Order::Ascending)
            .filter_map(|r| {
                let callout = r.unwrap();
                if !callouts.contains(&callout) {
                    Some(callout)
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
        {
            NODE_CALLOUT_RELATIONSHIP.remove(store, (node_id, &callout));
            CALLOUT_NODE_RELATIONSHIP.remove(store, (&callout, node_id));
        }
    }

    Ok((hashtags, callouts))
}
