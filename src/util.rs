use std::collections::HashSet;

use cosmwasm_std::{Order, Storage};

use crate::{
    error::ContractError,
    state::{
        models::NodeMetadata,
        storage::{
            HANDLE_NODE_RELATIONSHIP, NODE_HANDLE_RELATIONSHIP, NODE_ID_2_METADATA,
            NODE_ID_COUNTER, NODE_TAG_RELATIONSHIP, TAG_NODE_RELATIONSHIP,
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

pub fn process_tags_and_handles(
    store: &mut dyn Storage,
    node_id: u32,
    maybe_tags: Option<Vec<String>>,
    maybe_handles: Option<Vec<String>>,
    is_editing: bool,
) -> Result<(HashSet<String>, HashSet<String>), ContractError> {
    let mut tags: HashSet<String> = HashSet::with_capacity(2);
    let mut handles: HashSet<String> = HashSet::with_capacity(2);

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
    for token in maybe_handles.unwrap_or_default().iter() {
        // TODO: validate handle
        if let Some(handle) = token.strip_prefix("@") {
            let handle = handle.to_lowercase();
            if !handles.contains(&handle) {
                let handle = handle.to_owned();
                TAG_NODE_RELATIONSHIP.save(store, (&handle, node_id), &true)?;
                NODE_TAG_RELATIONSHIP.save(store, (node_id, &handle), &true)?;
                handles.insert(handle);
            }
        }
    }

    if is_editing {
        // Remove old tags and handles
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
        for handle in NODE_HANDLE_RELATIONSHIP
            .prefix(node_id)
            .keys(store, None, None, Order::Ascending)
            .filter_map(|r| {
                let handle = r.unwrap();
                if !handles.contains(&handle) {
                    Some(handle)
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
        {
            NODE_HANDLE_RELATIONSHIP.remove(store, (node_id, &handle));
            HANDLE_NODE_RELATIONSHIP.remove(store, (&handle, node_id));
        }
    }

    Ok((tags, handles))
}
