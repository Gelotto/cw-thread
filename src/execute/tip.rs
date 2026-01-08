use crate::{
    error::ContractError,
    state::{
        models::{TableMetadata, ROOT_ID},
        storage::{TIP_TOKEN_LUTAB, TOTAL_TIP_AMOUNTS},
    },
    util::load_node_metadata,
};
use cosmwasm_std::{attr, to_json_binary, Addr, Coin, Response, Storage, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;
use cw_lib::{
    models::{TokenAmountV2, TokenV2},
    utils::funds::{build_send_msg, has_funds},
};
use cw_table::msg::KeyValue;

use super::Context;

pub fn exec_tip(
    ctx: Context,
    token_amount: TokenAmountV2,
) -> Result<Response, ContractError> {
    let Context { deps, info, env } = ctx;

    // Load the thread's creator address
    let creator = load_node_metadata(deps.storage, ROOT_ID, true)?
        .unwrap()
        .created_by;

    // Forbid self-tipping
    if creator == info.sender {
        return Err(ContractError::NotAuthorized {
            reason: "Cannot tip yourself".to_owned(),
        });
    }

    // Validate and take payment
    let mut resp = match &token_amount.token {
        TokenV2::Denom(denom) => tip_native(
            deps.storage,
            &creator,
            &info.funds,
            denom,
            token_amount.amount,
        ),
        TokenV2::Address(cw20_addr) => {
            tip_cw20(deps.storage, &creator, cw20_addr, token_amount.amount)
        },
    }?;

    // If managed by a table contract, update its total tip amount index for
    // this token type.
    if let Some(table_update_msg) = increment_total_tip_amount(
        deps.storage,
        &info.sender,
        &env.contract.address,
        &token_amount,
    )? {
        resp = resp.add_message(table_update_msg);
    }

    Ok(resp)
}

fn tip_native(
    store: &mut dyn Storage,
    creator: &Addr,
    funds: &Vec<Coin>,
    denom: &String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if !TIP_TOKEN_LUTAB.has(store, denom) {
        return Err(ContractError::UnauthorizedTipToken {
            token: denom.clone(),
        });
    }
    if !has_funds(funds, amount.into(), denom) {
        return Err(ContractError::InsufficientFunds {
            details: format!("Expected {}{} for tip", amount.u128(), denom),
        });
    }

    Ok(Response::new()
        .add_message(build_send_msg(creator, denom, amount)?)
        .add_attributes(vec![
            attr("action", "tip"),
            attr("tip_amount", amount.to_string()),
            attr("tip_denom", denom),
        ]))
}

fn tip_cw20(
    store: &mut dyn Storage,
    creator: &Addr,
    cw20_addr: &Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // Check if this CW20 token is in the allowlist
    let token_key = cw20_addr.to_string();
    if !TIP_TOKEN_LUTAB.has(store, &token_key) {
        return Err(ContractError::UnauthorizedTipToken {
            token: token_key,
        });
    }

    // Update total tip amounts for this token
    TOTAL_TIP_AMOUNTS.update(store, &token_key, |maybe_total| -> Result<_, ContractError> {
        Ok(maybe_total.unwrap_or_default() + amount)
    })?;

    // Build CW20 transfer message
    let transfer_msg = Cw20ExecuteMsg::Transfer {
        recipient: creator.to_string(),
        amount,
    };

    let wasm_msg = WasmMsg::Execute {
        contract_addr: cw20_addr.to_string(),
        msg: to_json_binary(&transfer_msg)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(wasm_msg)
        .add_attributes(vec![
            attr("action", "tip"),
            attr("tip_amount", amount.to_string()),
            attr("tip_token", cw20_addr.to_string()),
        ]))
}

fn increment_total_tip_amount(
    store: &mut dyn Storage,
    initiator: &Addr,
    contract: &Addr,
    token_amount: &TokenAmountV2,
) -> Result<Option<WasmMsg>, ContractError> {
    let token_key = token_amount.token.get_key();
    let amount = token_amount.amount;
    let new_total = TOTAL_TIP_AMOUNTS.update(
        store,
        &token_key,
        |maybe_amount| -> Result<_, ContractError> {
            Ok(maybe_amount.unwrap_or_default() + amount)
        },
    )?;
    // build submsg for updating total tip amount table index?
    Ok(
        if let Some(table) = TableMetadata::load_client(store, contract)? {
            let index_name = format!("tip:{}", token_key);
            Some(table.index(
                initiator,
                vec![KeyValue::Uint128(index_name, Some(new_total))],
            )?)
        } else {
            None
        },
    )
}
