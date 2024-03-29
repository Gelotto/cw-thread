use crate::{
    error::ContractError,
    state::{
        models::{TableMetadata, ROOT_ID},
        storage::{NODE_ID_2_METADATA, TABLE},
        views::{load_mentions, load_tags},
    },
    util::load_node_metadata,
};
use cosmwasm_std::{attr, Addr, Response, Storage};
use cw_table::{
    client::Table,
    lifecycle::{LifecycleArgs, LifecycleSetupArgs},
    msg::{KeyValue, Relationship, RelationshipUpdates, TagUpdate, TagUpdates},
};

use super::Context;

pub const TABLE_INDEX_ACTIVITY_SCORE: &str = "activity";
pub const TABLE_INDEX_RANK: &str = "rank";

pub fn exec_setup(
    ctx: Context,
    args: LifecycleSetupArgs,
) -> Result<Response, ContractError> {
    let Context {
        deps, env, info, ..
    } = ctx;
    let resp = Response::new().add_attributes(vec![attr("action", "setup")]);
    let mut meta = load_node_metadata(deps.storage, ROOT_ID, true)?.unwrap();

    meta.created_by = args.initiator;
    NODE_ID_2_METADATA.save(deps.storage, meta.id, &meta)?;

    save_table_info(deps.storage, &info.sender, &args.id)?;

    let indices = vec![
        KeyValue::Int32(TABLE_INDEX_RANK.into(), Some(meta.rank)),
        KeyValue::Uint32(TABLE_INDEX_ACTIVITY_SCORE.into(), Some(0)),
    ];

    let relationships_to_add: Vec<Relationship> = vec![Relationship {
        address: meta.created_by.clone(),
        name: "creator".to_owned(),
        unique: false,
    }];

    let relationshps = RelationshipUpdates {
        remove: None,
        add: Some(relationships_to_add),
    };

    let tags = prepare_tag_updates(deps.storage, ROOT_ID)?;
    let table = Table::new(&info.sender, &env.contract.address);

    Ok(resp.add_message(table.update(
        &info.sender,
        Some(indices),
        Some(tags),
        Some(relationshps),
    )?))
}

fn save_table_info(
    store: &mut dyn Storage,
    table_addr: &Addr,
    contract_id: &String,
) -> Result<(), ContractError> {
    if TABLE.exists(store) {
        return Err(ContractError::NotAuthorized {
            reason: "Already setup".to_owned(),
        });
    }

    TABLE.save(
        store,
        &TableMetadata {
            address: table_addr.clone(),
            id: contract_id.clone(),
        },
    )?;

    Ok(())
}

fn prepare_tag_updates(
    store: &dyn Storage,
    node_id: u32,
) -> Result<TagUpdates, ContractError> {
    let mut tag_updates_to_add: Vec<TagUpdate> = load_mentions(store, node_id)?
        .iter()
        .map(|text| TagUpdate {
            text: text.clone(),
            unique: None,
        })
        .collect();

    tag_updates_to_add.append(
        &mut load_tags(store, node_id)?
            .iter()
            .map(|text| TagUpdate {
                text: text.clone(),
                unique: None,
            })
            .collect(),
    );

    Ok(TagUpdates {
        remove: None,
        add: Some(tag_updates_to_add),
    })
}

pub fn exec_teardown(
    _ctx: Context,
    _args: LifecycleArgs,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attributes(vec![attr("action", "teardown")]))
}

pub fn exec_suspend(
    _ctx: Context,
    _args: LifecycleArgs,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attributes(vec![attr("action", "suspend")]))
}

pub fn exec_resume(
    _ctx: Context,
    _args: LifecycleArgs,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attributes(vec![attr("action", "resume")]))
}
