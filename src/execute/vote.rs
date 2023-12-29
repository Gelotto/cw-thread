use crate::{
    error::ContractError,
    msg::NodeVoteMsg,
    state::{
        models::{NodeMetadata, NEGATIVE, NEUTRAL, POSITIVE},
        storage::{
            NEG_REPLY_RELATIONSHIP, NODE_ID_2_METADATA, NODE_ID_ADDR_2_SENTIMENT,
            POS_REPLY_RELATIONSHIP,
        },
    },
};
use cosmwasm_std::{attr, Response, Storage};

use super::Context;

pub fn exec_vote(
    ctx: Context,
    msg: NodeVoteMsg,
) -> Result<Response, ContractError> {
    let Context { deps, info, .. } = ctx;
    let curr_user_sentiment_u8 = msg.sentiment.to_u8();
    let node_id = msg.id;

    // Abort tx if duplicate vote (e.g. user up-votes same node twice)
    let mut prev_user_sentiment_u8 = NEUTRAL;
    NODE_ID_ADDR_2_SENTIMENT.update(
        deps.storage,
        (node_id, &info.sender),
        |maybe_sentiment_u8| -> Result<_, ContractError> {
            if let Some(senitment_u8) = maybe_sentiment_u8 {
                prev_user_sentiment_u8 = senitment_u8;
                if senitment_u8 == curr_user_sentiment_u8 {
                    return Err(ContractError::AlreadyVoted { node_id: node_id });
                }
            }
            Ok(curr_user_sentiment_u8)
        },
    )?;

    // Update the metadata of the node voted on and return the updated metadata
    // as well as the node's "previous" sentiment and rank prior to the update.
    // We use this below to update the node's relationships.
    let (meta, prev_sentiment_u8, prev_rank) = update_node_metadata(
        deps.storage,
        node_id,
        prev_user_sentiment_u8,
        curr_user_sentiment_u8,
    )?;

    if let Some(parent_id) = meta.parent_id {
        // Remove stale entry from previous reply relationship map
        (if prev_sentiment_u8 == POSITIVE {
            POS_REPLY_RELATIONSHIP
        } else {
            NEG_REPLY_RELATIONSHIP
        })
        .remove(deps.storage, (parent_id, prev_rank, node_id));
        // Insert new entry into new reply relationship map
        if curr_user_sentiment_u8 != NEUTRAL {
            (if curr_user_sentiment_u8 == POSITIVE {
                POS_REPLY_RELATIONSHIP
            } else {
                NEG_REPLY_RELATIONSHIP
            })
            .save(deps.storage, (parent_id, meta.rank, node_id), &true)?;
        }
    }

    Ok(Response::new().add_attributes(vec![
        attr("action", "vote"),
        attr("node_id", node_id.to_string()),
        attr("new_vote_sentiment", curr_user_sentiment_u8.to_string()),
        attr("old_vote_sentiment", prev_user_sentiment_u8.to_string()),
        attr("new_node_sentiment", meta.sentiment.to_string()),
        attr("old_node_sentiment", prev_sentiment_u8.to_string()),
    ]))
}

pub fn update_node_metadata(
    store: &mut dyn Storage,
    node_id: u32,
    prev_user_sentiment: u8,
    curr_user_sentiment: u8,
) -> Result<(NodeMetadata, u8, u32), ContractError> {
    let mut prev_node_sentiment = POSITIVE;
    let mut prev_n_votes = 0;

    let metadata = if prev_user_sentiment == NEUTRAL {
        // User is going from no or "neutral" sentiment to a postiive/negative one.
        NODE_ID_2_METADATA.update(
            store,
            node_id,
            |maybe_metadata| -> Result<_, ContractError> {
                if let Some(mut metadata) = maybe_metadata {
                    let curr_node_sentiment = metadata.sentiment;
                    let curr_node_sentiment_compl = if metadata.sentiment == POSITIVE {
                        NEGATIVE
                    } else {
                        POSITIVE
                    };
                    prev_node_sentiment = metadata.sentiment;
                    prev_n_votes = metadata.rank;
                    if curr_user_sentiment == curr_node_sentiment {
                        metadata.rank += 1;
                    } else {
                        if metadata.rank == 0 {
                            metadata.sentiment = curr_node_sentiment_compl;
                            metadata.rank += 1;
                        } else {
                            metadata.rank -= 1;
                        }
                    }
                    Ok(metadata)
                } else {
                    Err(ContractError::NodeNotFound { node_id })
                }
            },
        )?
    } else {
        // User has changed sentiment from positive to negative or vice versa
        NODE_ID_2_METADATA.update(
            store,
            node_id,
            |maybe_metadata| -> Result<_, ContractError> {
                if let Some(mut metadata) = maybe_metadata {
                    let curr_node_sentiment = metadata.sentiment;
                    let curr_node_sentiment_compl = if metadata.sentiment == POSITIVE {
                        NEGATIVE
                    } else {
                        POSITIVE
                    };
                    prev_node_sentiment = metadata.sentiment;
                    prev_n_votes = metadata.rank;
                    if curr_user_sentiment == curr_node_sentiment {
                        metadata.rank += 1;
                    } else {
                        // User switch sentiment
                        if metadata.rank >= 2 {
                            metadata.sentiment = curr_node_sentiment_compl;
                            metadata.rank -= 2;
                        } else if metadata.rank == 1 {
                            metadata.sentiment = curr_node_sentiment_compl;
                        } else {
                            // rank == 0
                            metadata.sentiment = curr_node_sentiment_compl;
                            metadata.rank += 1;
                        }
                    }
                    Ok(metadata)
                } else {
                    Err(ContractError::NodeNotFound { node_id })
                }
            },
        )?
    };

    Ok((metadata, prev_node_sentiment, prev_n_votes))
}
