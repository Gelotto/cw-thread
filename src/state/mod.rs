pub mod models;
pub mod storage;
pub mod views;

use cosmwasm_std::Response;
use cw_lib::models::Owner;

use crate::{
    error::ContractError, execute::Context, msg::InstantiateMsg, util::process_tags_and_mentions,
};

use self::{
    models::{Config, NodeMetadata, ROOT_ID},
    storage::{
        CONFIG, NODE_ID_2_BODY, NODE_ID_2_METADATA, NODE_ID_2_SECTION, NODE_ID_2_TITLE,
        NODE_ID_COUNTER, OWNER,
    },
};

/// Top-level initialization of contract state
pub fn init(
    ctx: Context,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let Context { deps, info, env } = ctx;

    if let Some(owner) = &msg.owner {
        deps.api.addr_validate(owner.to_addr().as_str())?;
    }

    CONFIG.save(deps.storage, &Config { is_archived: false })?;

    OWNER.save(
        deps.storage,
        &msg.owner
            .unwrap_or_else(|| Owner::Address(info.sender.clone())),
    )?;

    NODE_ID_COUNTER.save(deps.storage, &ROOT_ID)?;

    NODE_ID_2_BODY.save(deps.storage, ROOT_ID, &msg.body.unwrap_or_default())?;

    if let Some(title) = msg.title {
        NODE_ID_2_TITLE.save(deps.storage, ROOT_ID, &title)?;
    }

    // Save sections
    let mut n_sections: u8 = 0;
    for (i, section) in msg.sections.unwrap_or_default().iter().enumerate() {
        NODE_ID_2_SECTION.save(deps.storage, (ROOT_ID, i as u8), &section)?;
        n_sections += 1;
    }

    NODE_ID_2_METADATA.save(
        deps.storage,
        ROOT_ID,
        &NodeMetadata {
            id: ROOT_ID,
            created_at: env.block.time,
            updated_at: None,
            created_by: info.sender.clone(),
            parent_id: None,
            rank: 0,
            n_sections,
            n_replies: 0,
            n_flags: 0,
        },
    )?;

    process_tags_and_mentions(deps.storage, ROOT_ID, msg.tags, msg.mentions, false)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}
