use crate::{
    error::ContractError,
    msg::NodeVoteMsg,
    state::{
        models::{DOWN, NIL, UP},
        storage::{NODE_ID_2_METADATA, NODE_ID_ADDR_2_SENTIMENT, RANKED_CHILD_RELATIONSHIP},
    },
};
use cosmwasm_std::{attr, Response, Storage};

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
    let Context { deps, info, .. } = ctx;

    for msg in msgs.iter() {
        let mut prev_user_sentiment_u8 = NIL;
        let curr_user_sentiment_u8 = msg.sentiment.to_u8();
        let child_id = msg.id;

        // Get the sender's previous vote sentiment WRT to the voted node and
        // update it according to the new vote.
        NODE_ID_ADDR_2_SENTIMENT.update(
            deps.storage,
            (child_id, &info.sender),
            |maybe_sentiment_u8| -> Result<_, ContractError> {
                if let Some(stored_sentiment_u8) = maybe_sentiment_u8 {
                    prev_user_sentiment_u8 = stored_sentiment_u8;
                }
                Ok(curr_user_sentiment_u8)
            },
        )?;

        // Remove the entry if the existing up or down vote is being unset.
        if prev_user_sentiment_u8 == curr_user_sentiment_u8 {
            NODE_ID_ADDR_2_SENTIMENT.remove(deps.storage, (child_id, &info.sender));
        }

        // Update the metadata of the node voted on and return the updated metadata
        // as well as the node's "previous" sentiment and rank prior to the update.
        // We use this below to update the node's relationships.
        let (maybe_parent_id, prev_rank, curr_rank) = update_node_rank(
            deps.storage,
            child_id,
            prev_user_sentiment_u8,
            curr_user_sentiment_u8,
        )?;

        // Update ranking of voted node WRT its parent node
        if let Some(parent_id) = maybe_parent_id {
            RANKED_CHILD_RELATIONSHIP.remove(deps.storage, (parent_id, prev_rank, child_id));
            RANKED_CHILD_RELATIONSHIP.save(
                deps.storage,
                (parent_id, curr_rank, child_id),
                &true,
            )?;
        }

        // TODO: Prepare data for updating the thread's table if applicable
    }

    Ok(Response::new().add_attributes(vec![attr("action", "vote")]))
}

pub fn update_node_rank(
    store: &mut dyn Storage,
    node_id: u32,
    prev_user_sentiment: u8,
    curr_user_sentiment: u8,
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
                if prev_user_sentiment == curr_user_sentiment {
                    // Undo/untoggle existing vote
                    if curr_user_sentiment == UP {
                        meta.rank -= 1;
                    } else {
                        meta.rank += 1;
                    }
                } else {
                    // Set or update vote
                    let delta = if prev_user_sentiment == NIL { 1 } else { 2 };
                    if curr_user_sentiment == UP {
                        meta.rank += delta;
                    } else {
                        meta.rank -= delta;
                    }
                }
                meta.sentiment = if curr_rank >= 0 { UP } else { DOWN };
                curr_rank = meta.rank;
                Ok(meta)
            } else {
                Err(ContractError::NodeNotFound { node_id })
            }
        },
    )?;

    Ok((parent_id, prev_rank, curr_rank))
}
