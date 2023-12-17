use crate::error::ContractError;
use cosmwasm_std::{attr, Response};

use super::Context;

pub fn exec_delete_node(
    ctx: Context,
    id: u32,
) -> Result<Response, ContractError> {
    let Context { deps, .. } = ctx;
    // TODO: update parent metadata
    // TODO: remove body
    // TODO: remove attachments
    // TODO: remove tags
    // TODO: remove callouts
    Ok(Response::new().add_attributes(vec![attr("action", "delete")]))
}
