use crate::{error::ContractError, msg::ConfigResponse, state::views::ConfigView};

use super::ReadonlyContext;

pub fn query_config(ctx: ReadonlyContext) -> Result<ConfigResponse, ContractError> {
    let ReadonlyContext { deps, .. } = ctx;
    let config = ConfigView::load(deps.storage)?;
    Ok(ConfigResponse(config))
}
