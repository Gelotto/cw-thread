use std::marker::PhantomData;

use cosmwasm_std::{Addr, Order};
use cw_storage_plus::Bound;

use crate::{
    error::ContractError,
    msg::{NodeViewByTagPaginationResponse, NodeViewRepliesPaginationResponse},
    state::{
        models::{NEGATIVE, POSITIVE},
        storage::{
            CALLOUT_NODE_RELATIONSHIP, HASHTAG_NODE_RELATIONSHIP, NEG_REPLY_RELATIONSHIP,
            POS_REPLY_RELATIONSHIP,
        },
        views::NodeView,
    },
    util::load_node_metadata,
};

use super::ReadonlyContext;

pub enum TagWrapper {
    Hashtag(String),
    Callout(String),
}

pub fn query_nodes_by_id(
    ctx: ReadonlyContext,
    ids: Vec<u32>,
    sender: Option<Addr>,
) -> Result<Vec<NodeView>, ContractError> {
    let ReadonlyContext { deps, .. } = ctx;
    let mut nodes: Vec<NodeView> = Vec::with_capacity(ids.len());
    for id in ids.iter() {
        nodes.push(NodeView::load(deps.storage, *id, &sender)?);
    }
    Ok(nodes)
}

pub fn query_nodes_in_reply_to(
    ctx: ReadonlyContext,
    parent_id: u32,
    cursor: Option<(u8, u32, u32)>,
    sender: Option<Addr>,
) -> Result<NodeViewRepliesPaginationResponse, ContractError> {
    let ReadonlyContext { deps, .. } = ctx;
    let parent_metadata = load_node_metadata(deps.storage, parent_id, true)?.unwrap();
    let page_size = parent_metadata.n_replies.min(25) as usize;

    // Starting point for resuming pagination using the cursor:
    let mut cursor_sentiment = POSITIVE;
    let start = if let Some((sentiment, rank, child_id)) = cursor {
        cursor_sentiment = sentiment;
        Some(Bound::Exclusive(((parent_id, rank, child_id), PhantomData)))
    } else {
        None
    };

    let mut replies: Vec<NodeView> = Vec::with_capacity(page_size);
    let mut cursor: Option<(u8, u32, u32)> = None;

    if cursor_sentiment == POSITIVE {
        for result in POS_REPLY_RELATIONSHIP
            .keys(deps.storage, None, start.clone(), Order::Descending)
            .take(page_size)
        {
            let (_, rank, child_id) = result?;
            replies.push(NodeView::load(deps.storage, child_id, &sender)?);
            if replies.len() == page_size {
                cursor = Some((POSITIVE, rank, child_id))
            }
        }
    }

    if replies.len() < 25 {
        for result in NEG_REPLY_RELATIONSHIP
            .keys(deps.storage, None, start, Order::Descending)
            .take(25 - replies.len())
        {
            let (_, rank, child_id) = result?;
            replies.push(NodeView::load(deps.storage, child_id, &sender)?);
            if replies.len() == page_size {
                cursor = Some((NEGATIVE, rank, child_id))
            }
        }
    }

    Ok(NodeViewRepliesPaginationResponse {
        nodes: replies,
        cursor,
    })
}

pub fn query_ancestor_nodes(
    ctx: ReadonlyContext,
    start_node_id: u32,
    levels: Option<u8>,
    sender: Option<Addr>,
) -> Result<Vec<NodeView>, ContractError> {
    let ReadonlyContext { deps, .. } = ctx;
    let levels = levels.unwrap_or(1);
    let mut nodes: Vec<NodeView> = Vec::with_capacity(levels as usize);

    let start_node_metadata = load_node_metadata(deps.storage, start_node_id, true)?.unwrap();
    let mut maybe_parent_id = start_node_metadata.reply_to_id;

    for _ in 0..levels {
        if let Some(parent_id) = maybe_parent_id {
            let node = NodeView::load(deps.storage, parent_id, &sender)?;
            maybe_parent_id = node.metadata.reply_to_id.clone();
            nodes.push(node);
        } else {
            break;
        }
    }

    Ok(nodes)
}

pub fn query_nodes_by_tag_or_callout(
    ctx: ReadonlyContext,
    wrapped_tag: TagWrapper,
    cursor: Option<u32>,
    sender: Option<Addr>,
) -> Result<NodeViewByTagPaginationResponse, ContractError> {
    let ReadonlyContext { deps, .. } = ctx;
    let mut nodes: Vec<NodeView> = Vec::with_capacity(8);
    let start = if let Some(cursor_node_id) = cursor {
        Some(Bound::Exclusive((cursor_node_id, PhantomData)))
    } else {
        None
    };
    let (map, tag) = match wrapped_tag {
        TagWrapper::Hashtag(s) => (HASHTAG_NODE_RELATIONSHIP, s.to_lowercase()),
        TagWrapper::Callout(s) => (CALLOUT_NODE_RELATIONSHIP, s.to_lowercase()),
    };

    for result in map
        .prefix(&tag)
        .keys(deps.storage, None, start, Order::Descending)
        .take(25)
    {
        let node_id = result?;
        nodes.push(NodeView::load(deps.storage, node_id, &sender)?);
    }
    Ok(NodeViewByTagPaginationResponse {
        cursor: nodes.last().and_then(|u| Some(u.metadata.id)),
        nodes,
    })
}
