#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response,
    StdResult,
};
use cw2::set_contract_version;
use komple_types::module::Modules;
use komple_types::query::ResponseWrapper;
use komple_types::shared::HUB_ADDR_NAMESPACE;
use komple_utils::{
    query_collection_address, query_module_address, query_storage, query_token_owner,
};
use std::collections::HashMap;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, OwnershipMsg, QueryMsg};
use crate::state::{Config, CONFIG, PERMISSION_MODULE_ADDR};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:komple-ownership-permission-module";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = deps.api.addr_validate(&msg.admin)?;

    let config = Config {
        admin: admin.clone(),
    };
    CONFIG.save(deps.storage, &config)?;

    PERMISSION_MODULE_ADDR.save(deps.storage, &info.sender)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", admin.to_string())
        .add_attribute("permission_module_addr", info.sender.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Check { data } => execute_check(deps, env, info, data),
    }
}

pub fn execute_check(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    data: Binary,
) -> Result<Response, ContractError> {
    let hub_addr = query_hub_addr(&deps)?;
    let mint_module_addr = query_module_address(&deps.querier, &hub_addr, Modules::Mint)?;

    let msgs: Vec<OwnershipMsg> = from_binary(&data)?;

    // TODO: Check if HashMap makes sense in this context
    let mut collection_map: HashMap<u32, Addr> = HashMap::new();

    for ownership_msg in msgs {
        let collection_addr = match collection_map.contains_key(&ownership_msg.collection_id) {
            true => collection_map
                .get(&ownership_msg.collection_id)
                .unwrap()
                .clone(),
            false => {
                let collection_addr = query_collection_address(
                    &deps.querier,
                    &mint_module_addr,
                    &ownership_msg.collection_id,
                )?;
                collection_map.insert(ownership_msg.collection_id, collection_addr.clone());
                collection_addr
            }
        };

        let owner =
            query_token_owner(&deps.querier, &collection_addr, &ownership_msg.token_id).unwrap();
        if owner != ownership_msg.address {
            return Err(ContractError::InvalidOwnership {});
        }
    }

    Ok(Response::new()
        .add_event(Event::new("komple_framework").add_attribute("module", "ownership_permission")))
}

// Queries the hub address from permission modules storage
fn query_hub_addr(deps: &DepsMut) -> Result<Addr, ContractError> {
    let permission_addr = PERMISSION_MODULE_ADDR.load(deps.storage)?;
    let res = query_storage::<Addr>(&deps.querier, &permission_addr, &HUB_ADDR_NAMESPACE)?;
    Ok(res.unwrap())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ResponseWrapper<Config>> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ResponseWrapper {
        query: "config".to_string(),
        data: config,
    })
}
