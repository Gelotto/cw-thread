use crate::{
    error::ContractError,
    state::{
        is_action_authorized,
        models::{NodeMetadata, TableMetadata, ROOT_ID},
        storage::{
            ACTIVITY_SCORE, IX_CHILD, IX_MENTION_NODE, IX_NODE_MENTION, IX_RANKED_CHILD, IX_TAG_NODE,
            NODE_ID_2_BODY, NODE_ID_2_FLAG, NODE_ID_2_METADATA, NODE_ID_2_SECTION,
            NODE_ID_ADDR_2_SENTIMENT, TABLE,
        },
    },
    util::load_node_metadata,
};
use cosmwasm_std::{attr, Addr, Order, Response, Storage};
use cw_table::{client::Table, msg::KeyValue};

use super::Context;

/// Deletes a node and all its descendants from the thread.
///
/// If deleting the root node (id == ROOT_ID), purges all contract state and
/// removes the thread from its parent table contract.
///
/// For non-root nodes:
/// - Recursively collects all descendant node IDs using depth-first traversal
/// - Deletes descendants in reverse order (bottom-up) to maintain referential integrity
/// - Updates parent's reply count
/// - Updates activity score in table contract if applicable
///
/// Authorization: Only the node creator or contract owner can delete a node.
pub fn exec_delete_node(
    ctx: Context,
    id: u32,
) -> Result<Response, ContractError> {
    let Context { deps, env, info } = ctx;
    let node = load_node_metadata(deps.storage, id, true)?.unwrap();
    let maybe_table_metadata = TABLE.may_load(deps.storage)?;
    let mut resp = Response::new().add_attributes(vec![attr("action", "delete")]);

    if !(node.created_by == info.sender
        || is_action_authorized(&deps, &info.sender, "/thread/delete")?)
    {
        return Err(ContractError::NotAuthorized {
            reason: "Not authorized to delete the thread".to_owned(),
        });
    }

    if id == ROOT_ID {
        // Zero-out all contract storage
        purge_contract_state(deps.storage);
        // Remove from table
        if let Some(TableMetadata { address, .. }) = maybe_table_metadata {
            let table = Table::new(&address, &env.contract.address);
            resp = resp.add_message(table.delete()?);
        }
    } else {
        // Collect all descendant node IDs recursively
        let child_ids = collect_all_descendants(deps.storage, id)?;

        // Delete all descendants (bottom-up to avoid parent reference issues)
        for child_id in child_ids.iter().rev() {
            let child_node = load_node_metadata(deps.storage, *child_id, true)?.unwrap();
            delete_child_node(deps.storage, &child_node)?;
        }

        // Delete the node itself
        delete_child_node(deps.storage, &node)?;

        // Update table if applicable
        if let Some(TableMetadata { address, .. }) = maybe_table_metadata {
            let table = Table::new(&address, &env.contract.address);
            // Update activity score to reflect deletions
            let activity_score = ACTIVITY_SCORE.load(deps.storage)?;
            resp = resp.add_message(table.update(
                &info.sender,
                Some(vec![KeyValue::Uint32(
                    "activity".to_owned(),
                    Some(activity_score),
                )]),
                None,
                None,
            )?);
        }
    }

    Ok(resp)
}

/// Recursively collects all descendant node IDs using depth-first traversal.
///
/// Uses an iterative approach with a stack to avoid stack overflow on deep trees.
/// Returns all descendant IDs in order of discovery (not deletion order).
/// Caller should reverse the list for bottom-up deletion.
fn collect_all_descendants(
    store: &dyn Storage,
    parent_id: u32,
) -> Result<Vec<u32>, ContractError> {
    let mut all_descendants = Vec::new();
    let mut to_visit = vec![parent_id];

    while let Some(current_id) = to_visit.pop() {
        // Get direct children of current node
        let children: Vec<u32> = IX_CHILD
            .prefix(current_id)
            .keys(store, None, None, Order::Ascending)
            .map(|r| r.unwrap())
            .collect();

        // Add children to descendants list
        all_descendants.extend(children.iter());

        // Add children to visit queue for recursive traversal
        to_visit.extend(children);
    }

    Ok(all_descendants)
}

/// Purges all contract state by iterating through storage keys in batches.
///
/// Processes 20 keys at a time to avoid running out of gas on large datasets.
/// Used when deleting the root node to completely reset the contract.
fn purge_contract_state(storage: &mut dyn Storage) {
    loop {
        let keys: Vec<_> = storage
            .range_keys(None, None, Order::Ascending)
            .take(20)
            .collect();
        if keys.is_empty() {
            break;
        }
        for k in keys {
            storage.remove(&k);
        }
    }
}

/// Deletes a single node and all its associated data from storage.
///
/// Removes:
/// - Node metadata and body
/// - Parent-child relationships (IX_CHILD, IX_RANKED_CHILD)
/// - Sentiment/voting data
/// - Flags
/// - Sections
/// - Tags and mentions (both forward and reverse indices)
///
/// Also updates the parent's reply count if the node has a parent.
/// Uses node-scoped index queries for efficient O(M) deletion where M is
/// the data size for this node, rather than O(N) over all nodes.
fn delete_child_node(
    store: &mut dyn Storage,
    node: &NodeMetadata,
) -> Result<(), ContractError> {
    let id = node.id;

    // Remove metadata
    NODE_ID_2_METADATA.remove(store, id);

    if let Some(parent_id) = node.parent_id {
        // Remove child relationship
        IX_CHILD.remove(store, (parent_id, id));
        // Remove from ranked child ordering map
        IX_RANKED_CHILD.remove(store, (parent_id, node.rank, id));
        // Update parent metadata
        NODE_ID_2_METADATA.update(
            store,
            parent_id,
            |maybe_parent| -> Result<_, ContractError> {
                if let Some(mut parent) = maybe_parent {
                    parent.n_replies -= 1;
                    Ok(parent)
                } else {
                    Err(ContractError::NodeNotFound { node_id: parent_id })
                }
            },
        )?;
    }

    // Purge the node's sentiment state
    {
        let addrs: Vec<Addr> = NODE_ID_ADDR_2_SENTIMENT
            .prefix(id)
            .keys(store, None, None, Order::Ascending)
            .map(|r| r.unwrap())
            .collect();
        for addr in addrs.iter() {
            NODE_ID_ADDR_2_SENTIMENT.remove(store, (id, addr));
        }
    }

    // Remove node body text
    NODE_ID_2_BODY.remove(store, id);

    // Remove flagged addresses with respect to the node
    {
        let addrs: Vec<Addr> = NODE_ID_2_FLAG
            .prefix(id)
            .keys(store, None, None, Order::Ascending)
            .map(|r| r.unwrap())
            .collect();
        for addr in addrs.iter() {
            NODE_ID_2_FLAG.remove(store, (id, addr));
        }
    }

    // Remove sections
    for i in 0..node.n_sections {
        NODE_ID_2_SECTION.remove(store, (id, i));
    }

    // Remove tags (node-specific)
    {
        let tags: Vec<String> = IX_NODE_TAG
            .prefix(id)
            .keys(store, None, None, Order::Ascending)
            .map(|r| r.unwrap())
            .collect();
        for tag in tags.iter() {
            IX_NODE_TAG.remove(store, (id, tag));
            IX_TAG_NODE.remove(store, (tag, id));
        }
    }

    // Remove mentions (node-specific)
    {
        let mentions: Vec<String> = IX_NODE_MENTION
            .prefix(id)
            .keys(store, None, None, Order::Ascending)
            .map(|r| r.unwrap())
            .collect();
        for mention in mentions.iter() {
            IX_NODE_MENTION.remove(store, (id, mention));
            IX_MENTION_NODE.remove(store, (mention, id));
        }
    }
    Ok(())
}
