use cosmwasm_std::Storage;

use crate::{
    error::ContractError,
    state::{
        models::NodeMetadata,
        storage::{NODE_ID_2_METADATA, NODE_ID_COUNTER},
    },
};

pub fn load_node_metadata(
    store: &dyn Storage,
    id: u32,
    strict: bool,
) -> Result<Option<NodeMetadata>, ContractError> {
    let maybe_metadata = NODE_ID_2_METADATA.may_load(store, id)?;
    if strict && maybe_metadata.is_none() {
        return Err(ContractError::NodeNotFound { node_id: id });
    }
    Ok(maybe_metadata)
}

pub fn next_node_id(store: &mut dyn Storage) -> Result<u32, ContractError> {
    Ok(NODE_ID_COUNTER.update(store, |n| -> Result<_, ContractError> { Ok(n + 1) })?)
}
