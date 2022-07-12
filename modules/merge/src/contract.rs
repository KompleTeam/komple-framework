#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, WasmMsg,
};
use cw2::set_contract_version;

use rift_types::collection::Collections;
use rift_types::module::Modules;
use rift_utils::{check_admin_privileges, get_collection_address, get_module_address};

use mint_module::msg::ExecuteMsg as MintModuleExecuteMsg;
use token_contract::msg::ExecuteMsg as TokenExecuteMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MergeAction, MergeMsg, QueryMsg};
use crate::state::{Config, CONFIG, CONTROLLER_ADDR};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:minter";
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
            // } => execute_permission_merge(deps, env, info, permission_msg, merge_msg),
        } => unimplemented!(),
    }
}

fn execute_update_merge_lock(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    lock: bool,
) -> Result<Response, ContractError> {
    let controller_addr = CONTROLLER_ADDR.load(deps.storage)?;
    let mut config = CONFIG.load(deps.storage)?;

    check_admin_privileges(&info.sender, &config.admin, Some(&controller_addr), None)?;

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
    let controller_addr = CONTROLLER_ADDR.load(deps.storage)?;
    let mint_module_addr = get_module_address(&deps, &controller_addr, Modules::MintModule)?;
    let passcard_module_addr =
        get_module_address(&deps, &controller_addr, Modules::PasscardModule)?;

    let merge_msgs: Vec<MergeMsg> = from_binary(&msg)?;
    let mut msgs: Vec<CosmosMsg> = vec![];

    for merge_msg in merge_msgs {
        match merge_msg.action {
            MergeAction::Mint => match merge_msg.collection_type {
                Collections::Normal => {
                    let msg = MintModuleExecuteMsg::MintTo {
                        collection_id: merge_msg.collection_id,
                        recipient: merge_msg.owner.unwrap(),
                    };
                    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: mint_module_addr.to_string(),
                        msg: to_binary(&msg)?,
                        funds: info.funds.clone(),
                    }));
                }
                Collections::Passcard => Err(ContractError::MergeFailed {})?,
            },
            MergeAction::Burn => {
                let address: Addr;

                match merge_msg.collection_type {
                    Collections::Normal => {
                        address = get_collection_address(
                            &deps,
                            &mint_module_addr,
                            merge_msg.collection_id,
                        )?;
                    }
                    Collections::Passcard => {
                        address = get_collection_address(
                            &deps,
                            &passcard_module_addr,
                            merge_msg.collection_id,
                        )?;
                    }
                }

                let msg = TokenExecuteMsg::Burn {
                    token_id: merge_msg.token_id.unwrap().to_string(),
                };
                msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: address.to_string(),
                    msg: to_binary(&msg)?,
                    funds: info.funds.clone(),
                }));
            }
        }
    }

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "execute_merge"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}
