use crate::{
    error::ContractError,
    msg::NodeReplyMsg,
    state::{
        models::{NodeMetadata, POSITIVE},
        storage::{
            CHILD_RELATIONSHIP, NODE_ID_2_ATTACHMENT, NODE_ID_2_BODY, NODE_ID_2_METADATA,
            POS_REPLY_RELATIONSHIP,
        },
    },
    util::{next_node_id, process_hashtags_and_callouts},
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
        reply_to_id,
        attachments,
    } = msg;

    // TODO: Validate all data

    // Ensure the reply_to node exists and update its total reply counter
    NODE_ID_2_METADATA.update(
        deps.storage,
        reply_to_id,
        |maybe_parent| -> Result<_, ContractError> {
            if let Some(mut parent) = maybe_parent {
                parent.n_replies += 1;
                Ok(parent)
            } else {
                Err(ContractError::NodeNotFound {
                    node_id: reply_to_id,
                })
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
        reply_to_id: Some(reply_to_id),
        sentiment: POSITIVE,
        n_attachments,
        n_replies: 0,
        rank: 0,
        n_flags: 0,
    };

    NODE_ID_2_METADATA.save(deps.storage, child_id, &child_metadata)?;

    // Add to parent-child relationship
    CHILD_RELATIONSHIP.save(deps.storage, (reply_to_id, child_id), &true)?;

    // Add to ranked reply relationship
    POS_REPLY_RELATIONSHIP.save(deps.storage, (reply_to_id, 0, child_id), &true)?;

    process_hashtags_and_callouts(deps.storage, child_id, &body, false)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "reply"),
        attr("reply_to_id", reply_to_id.to_string()),
        attr("reply_id", child_id.to_string()),
    ]))
}
