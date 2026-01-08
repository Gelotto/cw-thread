use std::marker::PhantomData;

use cosmwasm_std::{Addr, Order};
use cw_storage_plus::Bound;

use crate::{
    error::ContractError,
    msg::{NodeViewByTagPaginationResponse, NodeViewRepliesPaginationResponse},
    state::{
        storage::{IX_MENTION_NODE, IX_RANKED_CHILD, IX_TAG_NODE},
        views::NodeView,
    },
    util::load_node_metadata,
};

use super::ReadonlyContext;

pub const DEFAULT_PAGINATION_LIMIT: u8 = 25;

pub enum TagWrapper {
    Tag(String),
    Mention(String),
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

pub fn query_child_nodes(
    ctx: ReadonlyContext,
    parent_id: u32,
    cursor: Option<(u32, i32, u32)>,
    limit: Option<u8>,
    sender: Option<Addr>,
) -> Result<NodeViewRepliesPaginationResponse, ContractError> {
    let ReadonlyContext { deps, .. } = ctx;
    let parent_metadata = load_node_metadata(deps.storage, parent_id, true)?.unwrap();
    let page_size = parent_metadata
        .n_replies
        .min(limit.unwrap_or(DEFAULT_PAGINATION_LIMIT) as u16)
        .min(DEFAULT_PAGINATION_LIMIT as u16) as usize;

    let start = if let Some(cursor) = cursor {
        Some(Bound::Exclusive((cursor, PhantomData)))
    } else {
        Some(Bound::Inclusive((
            (parent_id, i32::MAX, u32::MAX),
            PhantomData,
        )))
    };

    let stop = Some(Bound::Exclusive((
        (parent_id, i32::MIN, u32::MIN),
        PhantomData,
    )));

    let mut replies: Vec<NodeView> = Vec::with_capacity(page_size);
    let mut cursor: Option<(u32, i32, u32)> = None;

    for result in IX_RANKED_CHILD
        .keys(deps.storage, stop, start.clone(), Order::Descending)
        .take(page_size)
    {
        let (parent_id, rank, child_id) = result?;
        replies.push(NodeView::load(deps.storage, child_id, &sender)?);
        if replies.len() == page_size {
            cursor = Some((parent_id, rank, child_id))
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
    let mut maybe_parent_id = start_node_metadata.parent_id;

    for _ in 0..levels {
        if let Some(parent_id) = maybe_parent_id {
            let node = NodeView::load(deps.storage, parent_id, &sender)?;
            maybe_parent_id = node.metadata.parent_id.clone();
            nodes.push(node);
        } else {
            break;
        }
    }

    Ok(nodes)
}

pub fn query_nodes_by_tag_or_mention(
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
        TagWrapper::Tag(s) => (IX_TAG_NODE, s.to_lowercase()),
        TagWrapper::Mention(s) => (IX_MENTION_NODE, s.to_lowercase()),
    };

    for result in map
        .prefix(&tag)
        .keys(deps.storage, None, start, Order::Descending)
        .take(DEFAULT_PAGINATION_LIMIT as usize)
    {
        let node_id = result?;
        nodes.push(NodeView::load(deps.storage, node_id, &sender)?);
    }
    Ok(NodeViewByTagPaginationResponse {
        cursor: nodes.last().map(|u| u.metadata.id),
        nodes,
    })
}
