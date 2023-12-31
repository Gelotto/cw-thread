use crate::{
    error::ContractError,
    state::{
        models::UP,
        storage::{
            CHILD_RELATIONSHIP, DOWN_REPLY_RELATIONSHIP, MENTION_NODE_RELATIONSHIP,
            NODE_ID_2_ATTACHMENT, NODE_ID_2_BODY, NODE_ID_2_FLAG, NODE_ID_2_METADATA,
            NODE_ID_ADDR_2_SENTIMENT, NODE_MENTION_RELATIONSHIP, TAG_NODE_RELATIONSHIP,
            UP_REPLY_RELATIONSHIP,
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

    // TODO: replace some of these global maps with node-specific ones that we
    // can more simply call map.clear() on. The currenty method of iterating
    // over everything is O(N) and waiting to break.

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
        if node.sentiment == UP {
            UP_REPLY_RELATIONSHIP.remove(deps.storage, (parent_id, node.rank, id));
        } else {
            DOWN_REPLY_RELATIONSHIP.remove(deps.storage, (parent_id, node.rank, id));
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

    // Remove mentions
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
        let map = MENTION_NODE_RELATIONSHIP;
        let keys: Vec<_> = map
            .keys(deps.storage, None, None, Order::Ascending)
            .map(|r| r.unwrap())
            .collect();
        for (a, b) in keys.iter() {
            map.remove(deps.storage, (a, *b));
        }
        for (a, b) in keys.iter() {
            MENTION_NODE_RELATIONSHIP.remove(deps.storage, (a, *b));
        }
    }

    // Remove tags
    {
        let map = NODE_MENTION_RELATIONSHIP;
        let keys: Vec<_> = map
            .keys(deps.storage, None, None, Order::Ascending)
            .map(|r| r.unwrap())
            .collect();

        for (a, b) in keys.iter() {
            map.remove(deps.storage, (*a, b));
        }
    }
    {
        let map = MENTION_NODE_RELATIONSHIP;
        let keys: Vec<_> = map
            .keys(deps.storage, None, None, Order::Ascending)
            .map(|r| r.unwrap())
            .collect();

        for (a, b) in keys.iter() {
            map.remove(deps.storage, (a, *b));
        }
    }

    // TODO: Prepare data for updating the thread's table if applicable

    Ok(Response::new().add_attributes(vec![attr("action", "delete")]))
}
