use crate::{
    error::ContractError,
    msg::ConfigUpdateMsg,
    state::{
        models::ROOT_ID,
        storage::{CONFIG_TIP_TOKEN_ALLOWLIST, MAX_TIP_TOKEN_TYPES, TIP_TOKEN_LUTAB},
    },
    util::load_node_metadata,
};
use cosmwasm_std::{attr, Response};

use super::Context;

pub fn exec_set_config(
    ctx: Context,
    updates: ConfigUpdateMsg,
) -> Result<Response, ContractError> {
    let Context { deps, info, .. } = ctx;
    let node = load_node_metadata(deps.storage, ROOT_ID, true)?.unwrap();

    // Only thread owner can update config
    if info.sender != node.created_by {
        return Err(ContractError::NotAuthorized {
            reason: "You do not own the thread".to_owned(),
        });
    }

    // Update accepted tip token types
    if let Some(tokens) = &updates.tip_tokens {
        if tokens.len() > MAX_TIP_TOKEN_TYPES {
            return Err(ContractError::ValidationError {
                reason: format!("Max number of tip token types is {}", MAX_TIP_TOKEN_TYPES),
            });
        }
        CONFIG_TIP_TOKEN_ALLOWLIST.save(deps.storage, tokens)?;
        TIP_TOKEN_LUTAB.clear(deps.storage);
        for token in tokens.iter() {
            TIP_TOKEN_LUTAB.save(deps.storage, &token.get_key(), &true)?;
        }
    }

    Ok(Response::new().add_attributes(vec![attr("action", "set_config")]))
}
