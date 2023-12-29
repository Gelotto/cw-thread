use crate::{
    error::ContractError,
    state::{
        models::POSITIVE,
        storage::{
            CHILD_RELATIONSHIP, HANDLE_NODE_RELATIONSHIP, NEG_REPLY_RELATIONSHIP,
            NODE_HANDLE_RELATIONSHIP, NODE_ID_2_ATTACHMENT, NODE_ID_2_BODY, NODE_ID_2_FLAG,
            NODE_ID_2_METADATA, NODE_ID_ADDR_2_SENTIMENT, POS_REPLY_RELATIONSHIP,
            TAG_NODE_RELATIONSHIP,
        },
    },
    util::load_node_metadata,
};
use cosmwasm_std::{attr, Addr, Order, Response};

use super::Context;

pub fn exec_delete_node(
    ctx: Context,
    id: u32,
) -> Result<Response, ContractError> {
    let Context { deps, .. } = ctx;
    let node = load_node_metadata(deps.storage, id, true)?.unwrap();

    // Remove metadata
    NODE_ID_2_METADATA.remove(deps.storage, id);

    if let Some(parent_id) = node.parent_id {
        // Remove child relationship
        CHILD_RELATIONSHIP.remove(deps.storage, (parent_id, id));

        // Update parent metadata
        NODE_ID_2_METADATA.update(
            deps.storage,
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

        // Remove from ranked reply ordering maps
        if node.sentiment == POSITIVE {
            POS_REPLY_RELATIONSHIP.remove(deps.storage, (parent_id, node.rank, id));
        } else {
            NEG_REPLY_RELATIONSHIP.remove(deps.storage, (parent_id, node.rank, id));
        }
    }

    // Purge the node's sentiment state
    {
        let addrs: Vec<Addr> = NODE_ID_ADDR_2_SENTIMENT
            .prefix(id)
            .keys(deps.storage, None, None, Order::Ascending)
            .map(|r| r.unwrap())
            .collect();
        for addr in addrs.iter() {
            NODE_ID_ADDR_2_SENTIMENT.remove(deps.storage, (id, addr));
        }
    }

    // Remove node body text
    NODE_ID_2_BODY.remove(deps.storage, id);

    // Remove flagged addresses with respect to the node
    {
        let addrs: Vec<Addr> = NODE_ID_2_FLAG
            .prefix(id)
            .keys(deps.storage, None, None, Order::Ascending)
            .map(|r| r.unwrap())
            .collect();
        for addr in addrs.iter() {
            NODE_ID_2_FLAG.remove(deps.storage, (id, addr));
        }
    }

    // Remove attachments
    for i in 0..node.n_attachments {
        NODE_ID_2_ATTACHMENT.remove(deps.storage, (id, i));
    }

    // Remove handles
    {
        let map = TAG_NODE_RELATIONSHIP;
        let keys: Vec<_> = map
            .keys(deps.storage, None, None, Order::Ascending)
            .map(|r| r.unwrap())
            .collect();
        for (a, b) in keys.iter() {
            map.remove(deps.storage, (a, *b));
        }
    }
    {
        let map = HANDLE_NODE_RELATIONSHIP;
        let keys: Vec<_> = map
            .keys(deps.storage, None, None, Order::Ascending)
            .map(|r| r.unwrap())
            .collect();
        for (a, b) in keys.iter() {
            map.remove(deps.storage, (a, *b));
        }
        for (a, b) in keys.iter() {
            HANDLE_NODE_RELATIONSHIP.remove(deps.storage, (a, *b));
        }
    }

    // Remove tags
    {
        let map = NODE_HANDLE_RELATIONSHIP;
        let keys: Vec<_> = map
            .keys(deps.storage, None, None, Order::Ascending)
            .map(|r| r.unwrap())
            .collect();

        for (a, b) in keys.iter() {
            map.remove(deps.storage, (*a, b));
        }
    }
    {
        let map = HANDLE_NODE_RELATIONSHIP;
        let keys: Vec<_> = map
            .keys(deps.storage, None, None, Order::Ascending)
            .map(|r| r.unwrap())
            .collect();

        for (a, b) in keys.iter() {
            map.remove(deps.storage, (a, *b));
        }
    }

    Ok(Response::new().add_attributes(vec![attr("action", "delete")]))
}
