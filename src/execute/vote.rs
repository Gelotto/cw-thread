use crate::{
    error::ContractError,
    msg::NodeVoteMsg,
    state::{
        models::{TableMetadata, NIL, ROOT_ID, UP},
        storage::{IX_RANKED_CHILD, NODE_ID_2_METADATA, NODE_ID_ADDR_2_SENTIMENT, TABLE},
    },
};
use cosmwasm_std::{attr, Response, Storage};
use cw_table::{client::Table, msg::KeyValue};

use super::Context;

pub fn exec_vote(
    ctx: Context,
    msg: NodeVoteMsg,
) -> Result<Response, ContractError> {
    exec_votes(ctx, vec![msg])
}

pub fn exec_votes(
    ctx: Context,
    msgs: Vec<NodeVoteMsg>,
) -> Result<Response, ContractError> {
    let Context { deps, info, env } = ctx;
    let mut resp = Response::new().add_attributes(vec![attr("action", "vote")]);

    for msg in msgs.iter() {
        let child_id = msg.id;

        let new_user_sentiment_u8 = msg.sentiment.to_u8();
        let old_user_sentiment_u8 = NODE_ID_ADDR_2_SENTIMENT
            .may_load(deps.storage, (child_id, &info.sender))?
            .unwrap_or(NIL);

        // Get the sender's previous vote sentiment WRT to the voted node and
        // update it according to the new vote.
        if new_user_sentiment_u8 != old_user_sentiment_u8 {
            NODE_ID_ADDR_2_SENTIMENT.save(
                deps.storage,
                (child_id, &info.sender),
                &new_user_sentiment_u8,
            )?;
        } else {
            // Remove the entry if the existing up or down vote is being unset.
            NODE_ID_ADDR_2_SENTIMENT.remove(deps.storage, (child_id, &info.sender));
        }

        // Update the metadata of the node voted on and return the updated metadata
        // as well as the node's "previous" sentiment and rank prior to the update.
        // We use this below to update the node's relationships.
        let (maybe_parent_id, prev_rank, curr_rank) = update_node_rank(
            deps.storage,
            child_id,
            old_user_sentiment_u8,
            new_user_sentiment_u8,
        )?;

        // Update ranking of voted node WRT its parent node
        if let Some(parent_id) = maybe_parent_id {
            IX_RANKED_CHILD.remove(deps.storage, (parent_id, prev_rank, child_id));
            IX_RANKED_CHILD.save(deps.storage, (parent_id, curr_rank, child_id), &true)?;
        }

        // Prepare data for updating the thread's table if applicable
        if child_id == ROOT_ID {
            if let Some(TableMetadata { address, .. }) = TABLE.may_load(deps.storage)? {
                let table = Table::new(&address, &env.contract.address);
                resp = resp.add_message(table.update(
                    &info.sender,
                    Some(vec![KeyValue::Int32("rank".to_owned(), Some(curr_rank))]),
                    None,
                    None,
                )?);
            }
        }
    }

    Ok(resp)
}

pub fn update_node_rank(
    store: &mut dyn Storage,
    node_id: u32,
    old_user_sentiment: u8,
    new_user_sentiment: u8,
) -> Result<(Option<u32>, i32, i32), ContractError> {
    let mut parent_id: Option<u32> = None;
    let mut prev_rank = 0;
    let mut curr_rank = 0;

    NODE_ID_2_METADATA.update(
        store,
        node_id,
        |maybe_metadata| -> Result<_, ContractError> {
            if let Some(mut meta) = maybe_metadata {
                parent_id = meta.parent_id;
                prev_rank = meta.rank;
                if old_user_sentiment == new_user_sentiment {
                    // Undo/untoggle existing vote
                    if new_user_sentiment == UP {
                        meta.rank -= 1;
                    } else {
                        meta.rank += 1;
                    }
                } else {
                    // Set or update vote
                    let delta = if old_user_sentiment == NIL { 1 } else { 2 };
                    if new_user_sentiment == UP {
                        meta.rank += delta;
                    } else {
                        meta.rank -= delta;
                    }
                }
                curr_rank = meta.rank;
                Ok(meta)
            } else {
                Err(ContractError::NodeNotFound { node_id })
            }
        },
    )?;

    Ok((parent_id, prev_rank, curr_rank))
}
