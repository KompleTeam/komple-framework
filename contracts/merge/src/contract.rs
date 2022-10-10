use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MergeMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG, HUB_ADDR, OPERATORS};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo,
    Response, StdError, StdResult, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use cw721_base::msg::ExecuteMsg as Cw721ExecuteMsg;
use komple_mint_module::msg::ExecuteMsg as MintModuleExecuteMsg;
use komple_permission_module::msg::ExecuteMsg as PermissionExecuteMsg;
use komple_token_module::msg::ExecuteMsg as TokenExecuteMsg;
use komple_types::module::Modules;
use komple_types::query::ResponseWrapper;
use komple_utils::{check_admin_privileges, storage::StorageHelper};
use semver::Version;
use std::collections::HashMap;

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

    HUB_ADDR.save(deps.storage, &info.sender)?;

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
    let hub_addr = HUB_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let mut config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        hub_addr,
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
    let hub_addr = HUB_ADDR.load(deps.storage)?;
    let permission_module_addr =
        StorageHelper::query_module_address(&deps.querier, &hub_addr, Modules::Permission)?;

    let mut msgs: Vec<CosmosMsg> = vec![];

    let permission_msg = PermissionExecuteMsg::Check {
        module: Modules::Merge.to_string(),
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

fn execute_update_operators(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    addrs: Vec<String>,
) -> Result<Response, ContractError> {
    let hub_addr = HUB_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        hub_addr,
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

/// Constructs the mint and burn messages
fn make_merge_msg(
    deps: &DepsMut,
    info: &MessageInfo,
    msg: Binary,
    msgs: &mut Vec<CosmosMsg>,
) -> Result<(), ContractError> {
    let hub_addr = HUB_ADDR.load(deps.storage)?;
    let mint_module_addr =
        StorageHelper::query_module_address(&deps.querier, &hub_addr, Modules::Mint)?;

    // MergeMsg contains mint, burn and metadata infos
    let merge_msg: MergeMsg = from_binary(&msg)?;

    // Throw an error if there are no burn messages
    if merge_msg.burn.len() == 0 {
        return Err(ContractError::BurnNotFound {});
    }

    // Metadata length should be the same as mint messages
    if merge_msg.metadata_ids.is_some()
        && merge_msg.metadata_ids.as_ref().unwrap().len() != merge_msg.mint.len()
    {
        return Err(ContractError::InvalidMetadataIds {});
    }

    // Pushes the burn messages inside msgs list
    make_burn_messages(&deps, &info, &mint_module_addr, &merge_msg, msgs)?;

    // Pushes the mint messages inside msgs list
    make_mint_messages(deps, info, &mint_module_addr, &merge_msg, msgs)?;

    Ok(())
}

fn make_burn_messages(
    deps: &DepsMut,
    info: &MessageInfo,
    mint_module_addr: &Addr,
    merge_msg: &MergeMsg,
    msgs: &mut Vec<CosmosMsg>,
) -> Result<(), ContractError> {
    for burn_msg in &merge_msg.burn {
        let collection_addr = StorageHelper::query_collection_address(
            &deps.querier,
            &mint_module_addr,
            &burn_msg.collection_id,
        )?;

        let msg: Cw721ExecuteMsg<Empty, TokenExecuteMsg> = Cw721ExecuteMsg::Extension {
            msg: TokenExecuteMsg::Burn {
                token_id: burn_msg.token_id.to_string(),
            },
        };
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: collection_addr.to_string(),
            msg: to_binary(&msg)?,
            funds: info.funds.clone(),
        }));
    }
    Ok(())
}

fn make_mint_messages(
    deps: &DepsMut,
    info: &MessageInfo,
    mint_module_addr: &Addr,
    merge_msg: &MergeMsg,
    msgs: &mut Vec<CosmosMsg>,
) -> Result<(), ContractError> {
    let burn_collection_ids: Vec<u32> = merge_msg.burn.iter().map(|m| m.collection_id).collect();

    // Keeping the linked collections list inside a hashmap
    // Used for saving multiple queries on same collection id
    let mut linked_collection_map: HashMap<u32, Vec<u32>> = HashMap::new();

    for (index, collection_id) in merge_msg.mint.iter().enumerate() {
        let linked_collections = match linked_collection_map.contains_key(&collection_id) {
            true => linked_collection_map.get(&collection_id).unwrap().clone(),
            false => {
                let collections = StorageHelper::query_linked_collections(
                    &deps.querier,
                    &mint_module_addr,
                    *collection_id,
                )?;
                linked_collection_map.insert(*collection_id, collections.clone());
                collections
            }
        };

        // If there are some linked collections
        // They have to be in the burn message
        if linked_collections.len() > 0 {
            for linked_collection_id in linked_collections {
                if !burn_collection_ids.contains(&linked_collection_id) {
                    return Err(ContractError::LinkedCollectionNotFound {});
                }
            }
        }

        let msg = MintModuleExecuteMsg::MintTo {
            collection_id: *collection_id,
            recipient: info.sender.to_string(),
            metadata_id: merge_msg
                .metadata_ids
                .as_ref()
                .as_ref()
                .and_then(|ids| Some(ids[index])),
        };
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: mint_module_addr.to_string(),
            msg: to_binary(&msg)?,
            funds: info.funds.clone(),
        }));
    }

    Ok(())
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version: Version = CONTRACT_VERSION.parse()?;
    let contract_version: ContractVersion = get_contract_version(deps.storage)?;
    let storage_version: Version = contract_version.version.parse()?;

    if contract_version.contract != CONTRACT_NAME {
        return Err(
            StdError::generic_err("New version name should match the current version").into(),
        );
    }
    if storage_version >= version {
        return Err(
            StdError::generic_err("New version cannot be smaller than current version").into(),
        );
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}
