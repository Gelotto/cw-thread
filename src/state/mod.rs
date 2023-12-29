pub mod models;
pub mod storage;
pub mod views;

use cosmwasm_std::Response;
use cw_lib::models::Owner;

use crate::{
    error::ContractError, execute::Context, msg::InstantiateMsg, util::process_tags_and_handles,
};

use self::{
    models::{Config, NodeMetadata, POSITIVE},
    storage::{
        CONFIG, NODE_ID_2_ATTACHMENT, NODE_ID_2_BODY, NODE_ID_2_METADATA, NODE_ID_2_TITLE,
        NODE_ID_COUNTER, OWNER,
    },
};

/// Top-level initialization of contract state
pub fn init(
    ctx: Context,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let Context { deps, info, env } = ctx;
    let root_node_id: u32 = 0;

    if let Some(owner) = &msg.owner {
        deps.api.addr_validate(owner.to_addr().as_str())?;
    }

    CONFIG.save(deps.storage, &Config { is_archived: false })?;

    OWNER.save(
        deps.storage,
        &msg.owner
            .unwrap_or_else(|| Owner::Address(info.sender.clone())),
    )?;

    NODE_ID_COUNTER.save(deps.storage, &u32::MAX)?;

    NODE_ID_2_BODY.save(deps.storage, root_node_id, &msg.body)?;

    if let Some(title) = msg.title {
        NODE_ID_2_TITLE.save(deps.storage, root_node_id, &title)?;
    }

    // Save attachments
    let mut n_attachments: u8 = 0;
    for (i, attachment) in msg.attachments.unwrap_or_default().iter().enumerate() {
        NODE_ID_2_ATTACHMENT.save(deps.storage, (root_node_id, i as u8), &attachment)?;
        n_attachments += 1;
    }

    NODE_ID_2_METADATA.save(
        deps.storage,
        root_node_id,
        &NodeMetadata {
            id: root_node_id,
            created_at: env.block.time,
            updated_at: None,
            created_by: info.sender.clone(),
            parent_id: None,
            sentiment: POSITIVE,
            n_attachments,
            n_replies: 0,
            rank: 0,
            n_flags: 0,
        },
    )?;

    process_tags_and_handles(deps.storage, root_node_id, msg.tags, msg.handles, false)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}
