use crate::error::ContractError;
use crate::execute::flags::{exec_flag, exec_unflag};
use crate::execute::lifecycle::{exec_resume, exec_setup, exec_suspend, exec_teardown};
use crate::execute::reply::exec_reply;
use crate::execute::set_config::exec_set_config;
use crate::execute::vote::exec_vote;
use crate::execute::Context;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, NodesQueryMsg, QueryMsg};
use crate::query::info::query_thread_info;
use crate::query::nodes::{query_ancestor_nodes, query_nodes_by_id, query_nodes_in_reply_to};
use crate::query::ReadonlyContext;
use crate::state;
use cosmwasm_std::{entry_point, to_json_binary};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use cw_table::lifecycle::LifecycleExecuteMsg;

const CONTRACT_NAME: &str = "crates.io:cw-contract-template";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(state::init(Context { deps, env, info }, msg)?)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let ctx = Context { deps, env, info };
    match msg {
        ExecuteMsg::SetConfig(config) => exec_set_config(ctx, config),
        ExecuteMsg::Reply(msg) => exec_reply(ctx, msg),
        ExecuteMsg::Vote(msg) => exec_vote(ctx, msg),
        ExecuteMsg::Flag { id, reason } => exec_flag(ctx, id, reason),
        ExecuteMsg::Unflag { id } => exec_unflag(ctx, id),
        ExecuteMsg::Lifecycle(msg) => match msg {
            LifecycleExecuteMsg::Setup(args) => exec_setup(ctx, args),
            LifecycleExecuteMsg::Teardown(args) => exec_teardown(ctx, args),
            LifecycleExecuteMsg::Suspend(args) => exec_suspend(ctx, args),
            LifecycleExecuteMsg::Resume(args) => exec_resume(ctx, args),
        },
    }
}

#[entry_point]
pub fn query(
    deps: Deps,
    env: Env,
    msg: QueryMsg,
) -> Result<Binary, ContractError> {
    let ctx = ReadonlyContext { deps, env };
    let result = match msg {
        QueryMsg::Thread {} => to_json_binary(&query_thread_info(ctx)?),
        QueryMsg::Nodes(msg) => match msg {
            NodesQueryMsg::ById(ids) => to_json_binary(&query_nodes_by_id(ctx, ids)?),
            NodesQueryMsg::InReplyTo { id, cursor } => {
                to_json_binary(&query_nodes_in_reply_to(ctx, id, cursor)?)
            },
            NodesQueryMsg::AncestorsOf { id, levels } => {
                to_json_binary(&query_ancestor_nodes(ctx, id, levels)?)
            },
        },
    }?;
    Ok(result)
}

#[entry_point]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    _msg: MigrateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
