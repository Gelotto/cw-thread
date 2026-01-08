pub mod models;
pub mod storage;
pub mod views;

use cosmwasm_std::{Addr, DepsMut, Response};
use cw_acl::client::Acl;
use cw_lib::models::Owner;

use crate::{
    error::ContractError,
    execute::Context,
    msg::InstantiateMsg,
    util::process_tags_and_mentions,
    validation::{validate_body, validate_mentions, validate_sections, validate_tags, validate_title},
};

use self::{
    models::{NodeMetadata, ROOT_ID},
    storage::{
        ACTIVITY_SCORE, CONFIG_TIP_TOKEN_ALLOWLIST, NODE_ID_2_BODY, NODE_ID_2_METADATA,
        NODE_ID_2_SECTION, NODE_ID_2_TITLE, NODE_ID_COUNTER, N_TOTAL_REPLIES, OWNER,
        TIP_TOKEN_LUTAB,
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

    // Validate all input
    if let Some(ref title) = msg.title {
        validate_title(title)?;
    }
    if let Some(ref body) = msg.body {
        validate_body(body)?;
    }
    validate_tags(&msg.tags)?;
    validate_mentions(&msg.mentions)?;
    validate_sections(&msg.sections)?;

    CONFIG_TIP_TOKEN_ALLOWLIST.save(deps.storage, &msg.config.tip_tokens)?;
    ACTIVITY_SCORE.save(deps.storage, &0)?;
    N_TOTAL_REPLIES.save(deps.storage, &0)?;

    for token in msg.config.tip_tokens.iter() {
        TIP_TOKEN_LUTAB.save(deps.storage, &token.get_key(), &true)?;
    }

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
            depth: 0,
        },
    )?;

    process_tags_and_mentions(deps.storage, ROOT_ID, msg.tags, msg.mentions, false)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

pub fn authorize_action(
    deps: &DepsMut,
    principal: &Addr,
    action: &str,
) -> Result<(), ContractError> {
    if !is_action_authorized(deps, principal, action)? {
        Err(ContractError::NotAuthorized {
            reason: "Owner authorization required".to_owned(),
        })
    } else {
        Ok(())
    }
}

pub fn is_action_authorized(
    deps: &DepsMut,
    principal: &Addr,
    action: &str,
) -> Result<bool, ContractError> {
    Ok(match OWNER.load(deps.storage)? {
        Owner::Address(addr) => *principal == addr,
        Owner::Acl(acl_addr) => {
            let acl = Acl::new(&acl_addr);
            acl.is_allowed(&deps.querier, principal, action)?
        },
    })
}
