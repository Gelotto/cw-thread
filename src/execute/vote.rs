use crate::{
    error::ContractError,
    msg::NodeVoteMsg,
    state::{
        models::{NodeMetadata, NEGATIVE, POSITIVE},
        storage::{NEG_REPLY_RELATIONSHIP, NODE_ID_2_METADATA, POS_REPLY_RELATIONSHIP},
    },
};
use cosmwasm_std::{attr, Response, Storage};

use super::Context;

pub fn exec_vote(
    ctx: Context,
    msg: NodeVoteMsg,
) -> Result<Response, ContractError> {
    let Context { deps, .. } = ctx;
    let NodeVoteMsg {
        id: node_id,
        is_positive,
    } = msg;

    // Update the metadata of the node voted on and return the updated metadata
    // as well as the node's "previous" sentiment and rank prior to the update.
    // We use this below to update the node's relationships.
    let (meta, prev_sentiment, prev_rank) =
        update_node_metadata(deps.storage, node_id, is_positive)?;

    if let Some(parent_id) = meta.reply_to_id {
        // Remove stale entry from previous reply relationship map
        (if prev_sentiment == POSITIVE {
            POS_REPLY_RELATIONSHIP
        } else {
            NEG_REPLY_RELATIONSHIP
        })
        .remove(deps.storage, (parent_id, prev_rank, node_id));
        // Insert new entry into new reply relationship map
        (if meta.sentiment == POSITIVE {
            POS_REPLY_RELATIONSHIP
        } else {
            NEG_REPLY_RELATIONSHIP
        })
        .save(deps.storage, (parent_id, meta.rank, node_id), &true)?;
    }

    Ok(Response::new().add_attributes(vec![
        attr("action", "vote"),
        attr("node_id", node_id.to_string()),
        attr("is_positive", is_positive.to_string()),
    ]))
}

pub fn update_node_metadata(
    store: &mut dyn Storage,
    node_id: u32,
    is_positive: bool,
) -> Result<(NodeMetadata, u8, u32), ContractError> {
    let mut prev_sentiment = POSITIVE;
    let mut prev_n_votes = 0;
    Ok((
        NODE_ID_2_METADATA.update(
            store,
            node_id,
            |maybe_metadata| -> Result<_, ContractError> {
                if let Some(mut metadata) = maybe_metadata {
                    prev_sentiment = metadata.sentiment;
                    prev_n_votes = metadata.rank;
                    if metadata.sentiment == POSITIVE {
                        if is_positive {
                            metadata.rank += 1
                        } else {
                            metadata.sentiment = NEGATIVE;
                            if metadata.rank == 0 {
                                metadata.rank += 1;
                            } else {
                                metadata.rank -= 1;
                            }
                        }
                    } else {
                        if !is_positive {
                            metadata.rank += 1
                        } else {
                            metadata.sentiment = POSITIVE;
                            if metadata.rank == 0 {
                                metadata.rank += 1;
                            } else {
                                metadata.rank -= 1;
                            }
                        }
                    }
                    Ok(metadata)
                } else {
                    Err(ContractError::NodeNotFound { node_id })
                }
            },
        )?,
        prev_sentiment,
        prev_n_votes,
    ))
}
