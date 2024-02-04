use crate::{
    error::ContractError,
    msg::NodeEditMsg,
    state::storage::{NODE_ID_2_BODY, NODE_ID_2_METADATA, NODE_ID_2_SECTION, NODE_ID_2_TITLE},
    util::{load_node_metadata, process_tags_and_mentions},
};
use cosmwasm_std::{attr, Order, Response};

use super::Context;

pub fn exec_edit_node(
    ctx: Context,
    msg: NodeEditMsg,
) -> Result<Response, ContractError> {
    let Context { deps, env, .. } = ctx;
    let mut metadata = load_node_metadata(deps.storage, msg.id, true)?.unwrap();

    metadata.updated_at = Some(env.block.time);
    NODE_ID_2_METADATA.save(deps.storage, metadata.id, &metadata)?;

    if let Some(new_body) = &msg.body {
        process_tags_and_mentions(deps.storage, msg.id, msg.tags, msg.mentions, true)?;
        // TODO: validate new body
        NODE_ID_2_BODY.save(deps.storage, msg.id, new_body)?;
        if msg.title.is_some() {
            if metadata.parent_id.is_some() {
                return Err(ContractError::ValidationError {
                    reason: "Only the root node has a title".to_owned(),
                });
            } else {
                // TODO: validate new title
                let title = msg.title.unwrap();
                NODE_ID_2_TITLE.save(deps.storage, msg.id, &title)?;
            }
        }
    }

    if let Some(new_section) = &msg.sections {
        // TODO: validate sections
        // Remove old attachements
        for i in NODE_ID_2_SECTION
            .prefix(msg.id)
            .keys(deps.storage, None, None, Order::Ascending)
            .map(|r| r.unwrap())
            .collect::<Vec<u8>>()
        {
            NODE_ID_2_SECTION.remove(deps.storage, (msg.id, i as u8));
        }
        // Save new attachements
        for (i, section) in new_section.iter().enumerate() {
            NODE_ID_2_SECTION.save(deps.storage, (msg.id, i as u8), section)?;
        }
    }

    // TODO: Prepare data for updating the thread's table if applicable

    Ok(Response::new().add_attributes(vec![attr("action", "edit")]))
}
