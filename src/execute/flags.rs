use crate::{
    error::ContractError,
    state::{
        models::FlagMetadata,
        storage::{NODE_ID_2_FLAG, NODE_ID_2_METADATA},
    },
};
use cosmwasm_std::{attr, Response};

use super::Context;

pub fn exec_flag(
    ctx: Context,
    node_id: u32,
    maybe_reason: Option<String>,
) -> Result<Response, ContractError> {
    let Context { deps, info, env } = ctx;
    NODE_ID_2_METADATA.update(
        deps.storage,
        node_id,
        |maybe_metadata| -> Result<_, ContractError> {
            if let Some(mut metadata) = maybe_metadata {
                metadata.n_flags += 1;
                Ok(metadata)
            } else {
                Err(ContractError::NodeNotFound { node_id })
            }
        },
    )?;
    NODE_ID_2_FLAG.update(
        deps.storage,
        (node_id, &info.sender),
        |maybe_flag_metadata| -> Result<_, ContractError> {
            if maybe_flag_metadata.is_some() {
                return Err(ContractError::NotAuthorized {
                    reason: format!("Already flagged by {}", info.sender),
                });
            }
            Ok(FlagMetadata {
                flagged_at: env.block.time,
                flagged_by: info.sender.clone(),
                reason: maybe_reason,
            })
        },
    )?;
    Ok(Response::new().add_attributes(vec![attr("action", "flag")]))
}

pub fn exec_unflag(
    ctx: Context,
    node_id: u32,
) -> Result<Response, ContractError> {
    let Context { deps, info, .. } = ctx;
    let key = (node_id, &info.sender);
    if !NODE_ID_2_FLAG.has(deps.storage, key) {
        return Err(ContractError::NotAuthorized {
            reason: format!("Not flagged by {}", info.sender),
        });
    }
    NODE_ID_2_FLAG.remove(deps.storage, key);
    NODE_ID_2_METADATA.update(
        deps.storage,
        node_id,
        |maybe_metadata| -> Result<_, ContractError> {
            if let Some(mut metadata) = maybe_metadata {
                metadata.n_flags -= 1;
                Ok(metadata)
            } else {
                Err(ContractError::NodeNotFound { node_id })
            }
        },
    )?;
    Ok(Response::new().add_attributes(vec![attr("action", "unflag")]))
}
