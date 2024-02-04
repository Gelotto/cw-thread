use crate::{
    error::ContractError,
    state::{
        models::TableMetadata,
        storage::{IX_ADDR_SAVED_ID, NODE_ID_2_METADATA, TABLE},
    },
};
use cosmwasm_std::{attr, Response};
use cw_table::{
    client::Table,
    msg::{Relationship, RelationshipUpdates},
};

use super::Context;

pub fn exec_toggle_save(
    ctx: Context,
    is_saving: bool,
    node_ids: Vec<u32>,
) -> Result<Response, ContractError> {
    let Context { deps, info, env } = ctx;
    let maybe_table_metadata = TABLE.may_load(deps.storage)?;
    let using_table = maybe_table_metadata.is_some();

    // ID's of nodes that are newly saved. other ids are ignored
    let mut newly_saved_ids: Vec<u32> = if using_table && is_saving {
        Vec::with_capacity(node_ids.len())
    } else {
        vec![]
    };

    // Add addr + each node ID to ADDR_SAVED_TO index for use in determining if
    // a given address has saved a given node ID.
    for node_id in node_ids.iter() {
        if NODE_ID_2_METADATA.has(deps.storage, *node_id) {
            let ix_key = (&info.sender, *node_id);
            if is_saving {
                IX_ADDR_SAVED_ID.update(deps.storage, ix_key, |x| -> Result<_, ContractError> {
                    if x.is_none() && using_table {
                        newly_saved_ids.push(*node_id);
                    }
                    Ok(true)
                })?;
            } else {
                // => is unsaving
                IX_ADDR_SAVED_ID.remove(deps.storage, ix_key);
            }
        }
    }

    let mut resp = Response::new().add_attributes(vec![attr("action", "save")]);

    // For each saved node ID, add a relationship in the thread's table. The
    // relationship name encodes the node ID. For example, if the user saved
    // node ID 3, then the relationship name will be 'save:3'.
    //
    // If we're unsaving, we remove the relationships
    if let Some(TableMetadata { address, .. }) = maybe_table_metadata {
        let table = Table::new(&address, &env.contract.address);
        let relationship_updates = if is_saving {
            if newly_saved_ids.is_empty() {
                None
            } else {
                Some(RelationshipUpdates {
                    remove: None,
                    add: Some(
                        newly_saved_ids
                            .iter()
                            .map(|id| Relationship {
                                address: info.sender.clone(),
                                name: format!("save:{}", id),
                                unique: false,
                            })
                            .collect(),
                    ),
                })
            }
        } else {
            Some(RelationshipUpdates {
                add: None,
                remove: Some(
                    node_ids
                        .iter()
                        .map(|id| Relationship {
                            address: info.sender.clone(),
                            name: format!("save:{}", id),
                            unique: false,
                        })
                        .collect(),
                ),
            })
        };
        if relationship_updates.is_some() {
            resp =
                resp.add_message(table.update(&info.sender, None, None, relationship_updates)?);
        }
    }

    Ok(resp)
}
