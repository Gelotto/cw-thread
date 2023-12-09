use crate::{error::ContractError, state::models::Config};
use cosmwasm_std::{attr, Response};

use super::Context;

pub fn exec_reply(
    ctx: Context,
    config: Config,
) -> Result<Response, ContractError> {
    let Context { deps, .. } = ctx;
    Ok(Response::new().add_attributes(vec![attr("action", "reply")]))
}
