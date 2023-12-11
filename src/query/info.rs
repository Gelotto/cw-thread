use cw_acl::state::OWNER;

use crate::{
    error::ContractError,
    msg::ThreadInfoResponse,
    state::{
        storage::{CONFIG, TABLE},
        views::NodeView,
    },
};

use super::ReadonlyContext;

pub fn query_thread_info(ctx: ReadonlyContext) -> Result<ThreadInfoResponse, ContractError> {
    let ReadonlyContext { deps, .. } = ctx;
    let config = CONFIG.load(deps.storage)?;
    let owner = OWNER.load(deps.storage)?;
    let table_info = TABLE.may_load(deps.storage)?;
    Ok(ThreadInfoResponse {
        root: NodeView::load(deps.storage, 0)?,
        table: table_info,
        config,
        owner,
    })
}
