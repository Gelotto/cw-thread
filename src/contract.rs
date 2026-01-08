use crate::error::ContractError;
use crate::execute::delete_node::exec_delete_node;
use crate::execute::edit_node::exec_edit_node;
use crate::execute::flags::{exec_flag, exec_unflag};
use crate::execute::lifecycle::{exec_resume, exec_setup, exec_suspend, exec_teardown};
use crate::execute::reply::exec_reply;
use crate::execute::set_config::exec_set_config;
use crate::execute::tip::exec_tip;
use crate::execute::toggle_save::exec_toggle_save;
use crate::execute::vote::{exec_vote, exec_votes};
use crate::execute::Context;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, NodesQueryMsg, QueryMsg};
use crate::query::info::query_thread_info;
use crate::query::nodes::{
    query_ancestor_nodes, query_child_nodes, query_nodes_by_id, query_nodes_by_tag_or_mention,
    TagWrapper,
};
use crate::query::ReadonlyContext;
use crate::state;
use cosmwasm_std::{entry_point, to_json_binary};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use cw_table::lifecycle::LifecycleExecuteMsg;

const CONTRACT_NAME: &str = "crates.io:cw-thread";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Instantiates a new discussion thread contract.
///
/// Creates the root node of the thread hierarchy with an optional title, body,
/// sections, tags, and mentions. Validates all input and initializes the contract
/// state including owner, tip token allowlist, and activity tracking.
///
/// Only the owner (or authorized ACL principals) can perform privileged operations
/// like deleting any post or updating configuration.
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

/// Executes thread operations including creating replies, voting, editing,
/// deleting, tipping, flagging, and lifecycle management.
///
/// Operations are authorized based on the sender:
/// - Reply, Vote, Tip, Save, Flag: Any user
/// - Edit: Post creator only
/// - Delete: Post creator or contract owner
/// - SetConfig: Contract owner only
/// - Lifecycle (Setup/Teardown/Suspend/Resume): Table contract only
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let ctx = Context { deps, env, info };
    match msg {
        ExecuteMsg::SetConfig(updates) => exec_set_config(ctx, updates),
        ExecuteMsg::Reply(msg) => exec_reply(ctx, msg),
        ExecuteMsg::Vote(msg) => exec_vote(ctx, msg),
        ExecuteMsg::VoteMany(msgs) => exec_votes(ctx, msgs),
        ExecuteMsg::Edit(msg) => exec_edit_node(ctx, msg),
        ExecuteMsg::Tip(token_amount) => exec_tip(ctx, token_amount),
        ExecuteMsg::Save(ids) => exec_toggle_save(ctx, true, ids),
        ExecuteMsg::Unsave(ids) => exec_toggle_save(ctx, false, ids),
        ExecuteMsg::Delete { id } => exec_delete_node(ctx, id),
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

/// Queries thread data including thread info, nodes by ID, child nodes,
/// ancestor nodes, and nodes by tag or mention.
///
/// All queries are read-only and do not modify state. Some queries support
/// pagination for efficient data retrieval of large result sets.
///
/// Query results include node metadata, content, voting stats, and user-specific
/// data like save status and personal vote sentiment.
#[entry_point]
pub fn query(
    deps: Deps,
    env: Env,
    msg: QueryMsg,
) -> Result<Binary, ContractError> {
    let ctx = ReadonlyContext { deps, env };
    let result = match msg {
        QueryMsg::Thread { sender } => to_json_binary(&query_thread_info(ctx, sender)?),
        QueryMsg::Nodes(msg) => match msg {
            NodesQueryMsg::ByIds { ids, sender } => {
                to_json_binary(&query_nodes_by_id(ctx, ids, sender)?)
            },
            NodesQueryMsg::Children {
                id,
                cursor,
                limit,
                sender,
            } => to_json_binary(&query_child_nodes(ctx, id, cursor, limit, sender)?),
            NodesQueryMsg::WithTag {
                tag,
                cursor,
                sender,
            } => to_json_binary(&query_nodes_by_tag_or_mention(
                ctx,
                TagWrapper::Tag(tag),
                cursor,
                sender,
            )?),
            NodesQueryMsg::WithMention {
                mention,
                cursor,
                sender,
            } => to_json_binary(&query_nodes_by_tag_or_mention(
                ctx,
                TagWrapper::Mention(mention),
                cursor,
                sender,
            )?),
            NodesQueryMsg::Ancestors { id, levels, sender } => {
                to_json_binary(&query_ancestor_nodes(ctx, id, levels, sender)?)
            },
        },
    }?;
    Ok(result)
}

/// Migrates the contract to a new version.
///
/// Updates the contract version metadata stored on-chain. This is called
/// when upgrading the contract code to a new version.
#[entry_point]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    _msg: MigrateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
