use crate::{
    error::ContractError,
    state::{
        is_action_authorized,
        models::{NodeMetadata, TableMetadata, ROOT_ID},
        storage::{
            IX_CHILD, IX_MENTION_NODE, IX_NODE_MENTION, IX_RANKED_CHILD, IX_TAG_NODE,
            NODE_ID_2_BODY, NODE_ID_2_FLAG, NODE_ID_2_METADATA, NODE_ID_2_SECTION,
            NODE_ID_ADDR_2_SENTIMENT, TABLE,
        },
    },
    util::load_node_metadata,
};
use cosmwasm_std::{attr, Addr, Order, Response, Storage};
use cw_table::client::Table;

use super::Context;

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
        delete_child_node(deps.storage, &node)?;
        todo!("delete children and update table accordingly");
    }

    Ok(resp)
}

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

fn delete_child_node(
    store: &mut dyn Storage,
    node: &NodeMetadata,
) -> Result<(), ContractError> {
    // TODO: replace some of these global maps with node-specific ones that we
    // can more simply call map.clear() on. The currenty method of iterating
    // over everything is O(N) and waiting to break.
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

    // Remove mentions
    {
        let map = IX_TAG_NODE;
        let keys: Vec<_> = map
            .keys(store, None, None, Order::Ascending)
            .map(|r| r.unwrap())
            .collect();
        for (a, b) in keys.iter() {
            map.remove(store, (a, *b));
        }
    }
    {
        let map = IX_MENTION_NODE;
        let keys: Vec<_> = map
            .keys(store, None, None, Order::Ascending)
            .map(|r| r.unwrap())
            .collect();
        for (a, b) in keys.iter() {
            map.remove(store, (a, *b));
        }
        for (a, b) in keys.iter() {
            IX_MENTION_NODE.remove(store, (a, *b));
        }
    }

    // Remove tags
    {
        let map = IX_NODE_MENTION;
        let keys: Vec<_> = map
            .keys(store, None, None, Order::Ascending)
            .map(|r| r.unwrap())
            .collect();

        for (a, b) in keys.iter() {
            map.remove(store, (*a, b));
        }
    }
    {
        let map = IX_MENTION_NODE;
        let keys: Vec<_> = map
            .keys(store, None, None, Order::Ascending)
            .map(|r| r.unwrap())
            .collect();

        for (a, b) in keys.iter() {
            map.remove(store, (a, *b));
        }
    }
    Ok(())
}
