#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, WasmMsg,
};
use cw2::set_contract_version;
use std::collections::HashMap;

use komple_types::module::Modules;
use komple_types::query::ResponseWrapper;
use komple_utils::{
    check_admin_privileges, query_collection_address, query_linked_collections,
    query_module_address,
};

use mint_module::msg::ExecuteMsg as MintModuleExecuteMsg;
use permission_module::msg::ExecuteMsg as PermissionExecuteMsg;
use token_contract::msg::ExecuteMsg as TokenExecuteMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MergeMsg, QueryMsg};
use crate::state::{Config, CONFIG, CONTROLLER_ADDR, OPERATORS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:komple-merge-module";
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
        admin,
        merge_lock: false,
    };
    CONFIG.save(deps.storage, &config)?;

    CONTROLLER_ADDR.save(deps.storage, &info.sender)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateMergeLock { lock } => execute_update_merge_lock(deps, env, info, lock),
        ExecuteMsg::Merge { msg } => execute_merge(deps, env, info, msg),
        ExecuteMsg::PermissionMerge {
            permission_msg,
            merge_msg,
        } => execute_permission_merge(deps, env, info, permission_msg, merge_msg),
        ExecuteMsg::UpdateOperators { addrs } => execute_update_operators(deps, env, info, addrs),
    }
}

fn execute_update_merge_lock(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    lock: bool,
) -> Result<Response, ContractError> {
    let controller_addr = CONTROLLER_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let mut config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        controller_addr,
        operators,
    )?;

    config.merge_lock = lock;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "execute_update_merge_lock")
        .add_attribute("merge_lock", lock.to_string()))
}

fn execute_merge(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: Binary,
) -> Result<Response, ContractError> {
    let mut msgs: Vec<CosmosMsg> = vec![];

    make_merge_msg(&deps, &info, msg, &mut msgs)?;

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "execute_merge"))
}

fn execute_permission_merge(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    permission_msg: Binary,
    merge_msg: Binary,
) -> Result<Response, ContractError> {
    let controller_addr = CONTROLLER_ADDR.load(deps.storage)?;
    let permission_module_addr =
        query_module_address(&deps.querier, &controller_addr, Modules::PermissionModule)?;

    let mut msgs: Vec<CosmosMsg> = vec![];

    let permission_msg = PermissionExecuteMsg::Check {
        module: Modules::MergeModule,
        msg: permission_msg,
    };
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: permission_module_addr.to_string(),
        msg: to_binary(&permission_msg)?,
        funds: info.funds.clone(),
    }));

    make_merge_msg(&deps, &info, merge_msg, &mut msgs)?;

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "execute_permission_merge"))
}

fn make_merge_msg(
    deps: &DepsMut,
    info: &MessageInfo,
    msg: Binary,
    msgs: &mut Vec<CosmosMsg>,
) -> Result<(), ContractError> {
    let controller_addr = CONTROLLER_ADDR.load(deps.storage)?;
    let mint_module_addr =
        query_module_address(&deps.querier, &controller_addr, Modules::MintModule)?;

    let merge_msg: MergeMsg = from_binary(&msg)?;

    if merge_msg.burn.len() == 0 {
        return Err(ContractError::BurnNotFound {});
    }

    let mut burn_collection_ids: Vec<u32> = vec![];

    for burn_msg in merge_msg.clone().burn {
        burn_collection_ids.push(burn_msg.collection_id);

        let collection_addr =
            query_collection_address(&deps.querier, &mint_module_addr, &burn_msg.collection_id)?;

        let msg = TokenExecuteMsg::Burn {
            token_id: burn_msg.token_id.to_string(),
        };
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: collection_addr.to_string(),
            msg: to_binary(&msg)?,
            funds: info.funds.clone(),
        }));
    }

    let mut linked_collection_map: HashMap<u32, Vec<u32>> = HashMap::new();

    for collection_id in merge_msg.mint {
        let linked_collections = match linked_collection_map.contains_key(&collection_id) {
            true => linked_collection_map.get(&collection_id).unwrap().clone(),
            false => {
                let collections =
                    query_linked_collections(&deps.querier, &mint_module_addr, collection_id)?;
                linked_collection_map.insert(collection_id, collections.clone());
                collections
            }
        };

        if linked_collections.len() > 0 {
            for linked_collection_id in linked_collections {
                if !burn_collection_ids.contains(&linked_collection_id) {
                    return Err(ContractError::LinkedCollectionNotFound {});
                }
            }
        }

        let msg = MintModuleExecuteMsg::MintTo {
            collection_id,
            recipient: info.sender.to_string(),
        };
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: mint_module_addr.to_string(),
            msg: to_binary(&msg)?,
            funds: info.funds.clone(),
        }));
    }

    Ok(())
}

fn execute_update_operators(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    addrs: Vec<String>,
) -> Result<Response, ContractError> {
    let controller_addr = CONTROLLER_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        controller_addr,
        operators,
    )?;

    let addrs = addrs
        .iter()
        .map(|addr| -> StdResult<Addr> {
            let addr = deps.api.addr_validate(addr)?;
            Ok(addr)
        })
        .collect::<StdResult<Vec<Addr>>>()?;

    OPERATORS.save(deps.storage, &addrs)?;

    Ok(Response::new().add_attribute("action", "execute_update_operators"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Operators {} => to_binary(&query_operators(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ResponseWrapper<Config>> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ResponseWrapper::new("config", config))
}

fn query_operators(deps: Deps) -> StdResult<ResponseWrapper<Vec<String>>> {
    let addrs = OPERATORS.load(deps.storage)?;
    Ok(ResponseWrapper::new(
        "operators",
        addrs.iter().map(|addr| addr.to_string()).collect(),
    ))
}
