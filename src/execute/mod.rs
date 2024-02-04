pub mod delete_node;
pub mod edit_node;
pub mod flags;
pub mod lifecycle;
pub mod reply;
pub mod set_config;
pub mod tip;
pub mod toggle_save;
pub mod vote;

use cosmwasm_std::{DepsMut, Env, MessageInfo};

pub struct Context<'a> {
    pub deps: DepsMut<'a>,
    pub env: Env,
    pub info: MessageInfo,
}
