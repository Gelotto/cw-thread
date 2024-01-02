use crate::{
    error::ContractError,
    msg::NodeReplyMsg,
    state::{
        models::{NodeMetadata, UP},
        storage::{
            CHILD_RELATIONSHIP, NODE_ID_2_ATTACHMENT, NODE_ID_2_BODY, NODE_ID_2_METADATA,
            RANKED_CHILD_RELATIONSHIP,
        },
    },
    util::{next_node_id, process_tags_and_mentions},
};
use cosmwasm_std::{attr, Response};

use super::Context;

pub fn exec_reply(
    ctx: Context,
    msg: NodeReplyMsg,
) -> Result<Response, ContractError> {
    let Context { deps, env, info } = ctx;
    let NodeReplyMsg {
        body,
        parent_id,
        attachments,
        mentions,
        tags,
    } = msg;

    // TODO: Validate all data

    // Ensure the parent_id node exists and update its total reply counter
    NODE_ID_2_METADATA.update(
        deps.storage,
        parent_id,
        |maybe_parent| -> Result<_, ContractError> {
            if let Some(mut parent) = maybe_parent {
                parent.n_replies += 1;
                Ok(parent)
            } else {
                Err(ContractError::NodeNotFound { node_id: parent_id })
            }
        },
    )?;

    // generate the next node ID for the new reply node
    let child_id = next_node_id(deps.storage)?;

    // Save the reply's main html body
    NODE_ID_2_BODY.save(deps.storage, child_id, &body)?;

    // Save attachments
    let mut n_attachments: u8 = 0;
    for (i, attachment) in attachments.unwrap_or_default().iter().enumerate() {
        NODE_ID_2_ATTACHMENT.save(deps.storage, (child_id, i as u8), &attachment)?;
        n_attachments += 1;
    }

    // Build and save the reply node's metadata
    let child_metadata = NodeMetadata {
        id: child_id,
        created_at: env.block.time,
        updated_at: None,
        created_by: info.sender.clone(),
        parent_id: Some(parent_id),
        sentiment: UP,
        n_attachments,
        n_replies: 0,
        rank: 0,
        n_flags: 0,
    };

    NODE_ID_2_METADATA.save(deps.storage, child_id, &child_metadata)?;

    // Add to parent-child relationship
    CHILD_RELATIONSHIP.save(deps.storage, (parent_id, child_id), &true)?;

    // Add to ranked reply relationship
    RANKED_CHILD_RELATIONSHIP.save(deps.storage, (parent_id, 0, child_id), &true)?;

    process_tags_and_mentions(deps.storage, child_id, tags, mentions, false)?;

    // TODO: Prepare data for updating the thread's table if applicable

    Ok(Response::new().add_attributes(vec![
        attr("action", "reply"),
        attr("parent_id", parent_id.to_string()),
        attr("reply_id", child_id.to_string()),
    ]))
}
