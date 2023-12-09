use crate::{
    error::ContractError,
    msg::NodeVoteMsg,
    state::{
        models::{NEGATIVE_SENTIMENT, POSITIVE_SENTIMENT},
        storage::{IX_RANKED_REPLIES, NODE_ID_2_METADATA},
    },
};
use cosmwasm_std::{attr, Response};

use super::Context;

pub fn exec_vote(
    ctx: Context,
    msg: NodeVoteMsg,
) -> Result<Response, ContractError> {
    let Context { deps, .. } = ctx;
    let NodeVoteMsg {
        node_id,
        is_up_vote,
    } = msg;

    let mut prev_sentiment = POSITIVE_SENTIMENT;
    let mut prev_n_votes = 0;

    // Ensure the reply_to node exists and update its total reply counter
    let metadata = NODE_ID_2_METADATA.update(
        deps.storage,
        node_id,
        |maybe_metadata| -> Result<_, ContractError> {
            if let Some(mut metadata) = maybe_metadata {
                prev_sentiment = metadata.sentiment;
                prev_n_votes = metadata.n_votes;
                if metadata.sentiment == POSITIVE_SENTIMENT {
                    if is_up_vote {
                        metadata.n_votes += 1
                    } else {
                        if metadata.n_votes == 0 {
                            metadata.sentiment = NEGATIVE_SENTIMENT;
                            metadata.n_votes += 1;
                        }
                    }
                } else {
                    if !is_up_vote {
                        metadata.n_votes += 1
                    } else {
                        if metadata.n_votes == 0 {
                            metadata.sentiment = POSITIVE_SENTIMENT;
                            metadata.n_votes += 1;
                        }
                    }
                }
                Ok(metadata)
            } else {
                Err(ContractError::NodeNotFound { node_id })
            }
        },
    )?;

    if let Some(parent_id) = metadata.reply_to_id {
        let old_key = (parent_id, (prev_sentiment, prev_n_votes, node_id));
        let new_key = (parent_id, (metadata.sentiment, metadata.n_votes, node_id));
        IX_RANKED_REPLIES.remove(deps.storage, old_key);
        IX_RANKED_REPLIES.save(deps.storage, new_key, &true)?;
    }

    Ok(Response::new().add_attributes(vec![
        attr("action", "vote"),
        attr("node_id", node_id.to_string()),
        attr("is_up_vote", is_up_vote.to_string()),
    ]))
}
