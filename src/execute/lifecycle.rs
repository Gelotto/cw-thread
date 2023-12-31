use crate::{
    error::ContractError,
    state::{models::TableInfo, storage::TABLE},
};
use cosmwasm_std::{attr, Response};
use cw_table::lifecycle::{LifecycleArgs, LifecycleSetupArgs};

use super::Context;

pub fn exec_setup(
    ctx: Context,
    args: LifecycleSetupArgs,
) -> Result<Response, ContractError> {
    let Context { deps, info, .. } = ctx;
    let resp = Response::new().add_attributes(vec![attr("action", "setup")]);

    if TABLE.exists(deps.storage) {
        return Err(ContractError::NotAuthorized {
            reason: "Already setup".to_owned(),
        });
    }

    TABLE.save(
        deps.storage,
        &TableInfo {
            address: info.sender.clone(),
            id: args.id.clone(),
        },
    )?;

    // // Initialize user contract's data in parent table
    // if let Some(table_addr) = TABLE.load(deps.storage)? {
    //     if args.table != table_addr {
    //         return Err(ContractError::NotAuthorized {
    //             reason: "LifecycleMsg table not authorized".to_owned(),
    //         });
    //     }

    //     let table = Table::new(&table_addr, &env.contract.address);
    //     let owners = OWNERS.load(deps.storage)?;
    //     let profile = PROFILE.load(deps.storage)?;

    //     let mut indices = vec![
    //         KeyValue::Timestamp("created_at".into(), Some(profile.created_at)),
    //         KeyValue::String("created_by".into(), Some(info.sender.clone().into())),
    //         KeyValue::String("mention".into(), Some(profile.mention.clone())),
    //     ];

    //     if let Some(email) = &profile.email {
    //         indices.push(KeyValue::String("email".into(), Some(email.clone())));
    //     }

    //     let mut relationships: Vec<Relationship> = owners
    //         .iter()
    //         .map(|addr| Relationship {
    //             address: addr.clone(),
    //             name: "owner".to_owned(),
    //             unique: true,
    //         })
    //         .collect();

    //     if let Some(referrer) = REFERRER.may_load(deps.storage)? {
    //         relationships.push(Relationship {
    //             name: "referrer".into(),
    //             address: referrer,
    //             unique: false,
    //         });
    //     }

    //     let tags = TagUpdates {
    //         remove: None,
    //         add: Some(vec![TagUpdate {
    //             text: format!("@{}", profile.mention.to_lowercase()),
    //             unique: Some(true),
    //         }]),
    //     };

    //     resp = resp.add_message(table.update(
    //         &info.sender,
    //         Some(indices),
    //         Some(tags),
    //         Some(RelationshipUpdates {
    //             remove: None,
    //             add: Some(relationships),
    //         }),
    //     )?);
    // }

    Ok(resp)
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
