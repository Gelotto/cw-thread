use cosmwasm_std::Addr;
use cw_acl::state::OWNER;
use cw_lib::models::TokenAmountV2;

use crate::{
    error::ContractError,
    msg::ThreadInfoResponse,
    state::{
        storage::{N_TOTAL_REPLIES, TABLE, TOTAL_TIP_AMOUNTS},
        views::{ConfigView, NodeView},
    },
};

use super::ReadonlyContext;

pub fn query_thread_info(
    ctx: ReadonlyContext,
    sender: Option<Addr>,
) -> Result<ThreadInfoResponse, ContractError> {
    let ReadonlyContext { deps, .. } = ctx;
    let config = ConfigView::load(deps.storage)?;
    let owner = OWNER.load(deps.storage)?;
    let table_info = TABLE.may_load(deps.storage)?;
    let mut tip_token_amounts: Vec<TokenAmountV2> = Vec::with_capacity(config.tip_tokens.len());

    for token in &config.tip_tokens {
        let key = token.get_key();
        if let Some(amount) = TOTAL_TIP_AMOUNTS.may_load(deps.storage, &key)? {
            tip_token_amounts.push(TokenAmountV2 {
                token: token.clone(),
                amount,
            });
        }
    }

    Ok(ThreadInfoResponse {
        n_total_replies: N_TOTAL_REPLIES.load(deps.storage)?,
        tips: tip_token_amounts,
        root: NodeView::load(deps.storage, 0, &sender)?,
        table: table_info,
        config,
        owner,
    })
}
