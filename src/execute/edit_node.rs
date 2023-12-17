use crate::{
    error::ContractError,
    msg::NodeEditMsg,
    state::storage::{NODE_ID_2_ATTACHMENT, NODE_ID_2_BODY},
    util::process_hashtags_and_callouts,
};
use cosmwasm_std::{attr, Order, Response};

use super::Context;

pub fn exec_edit_node(
    ctx: Context,
    msg: NodeEditMsg,
) -> Result<Response, ContractError> {
    let Context { deps, .. } = ctx;

    if let Some(new_body) = &msg.body {
        // TODO: validate new body
        NODE_ID_2_BODY.save(deps.storage, msg.id, new_body)?;
        process_hashtags_and_callouts(deps.storage, msg.id, new_body, true)?;
    }

    if let Some(new_attachments) = &msg.attachments {
        // TODO: validate attachments
        // Remove old attachements
        for i in NODE_ID_2_ATTACHMENT
            .prefix(msg.id)
            .keys(deps.storage, None, None, Order::Ascending)
            .map(|r| r.unwrap())
            .collect::<Vec<u8>>()
        {
            NODE_ID_2_ATTACHMENT.remove(deps.storage, (msg.id, i as u8));
        }
        // Save new attachements
        for (i, attachment) in new_attachments.iter().enumerate() {
            NODE_ID_2_ATTACHMENT.save(deps.storage, (msg.id, i as u8), attachment)?;
        }
    }

    Ok(Response::new().add_attributes(vec![attr("action", "edit")]))
}
