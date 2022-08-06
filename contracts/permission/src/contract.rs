#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use komple_types::module::Modules;
use komple_types::permission::Permissions;
use komple_types::query::ResponseWrapper;
use komple_utils::{
    check_admin_privileges, query_collection_address, query_module_address, query_token_owner,
};
use std::collections::HashMap;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, OwnershipMsg, PermissionCheckMsg, QueryMsg};
use crate::state::{Config, CONFIG, CONTROLLER_ADDR, MODULE_PERMISSIONS, OPERATORS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:komple-permission-module";
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

    let config = Config { admin };
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
        ExecuteMsg::UpdateModulePermissions {
            module,
            permissions,
        } => execute_update_module_permissions(deps, env, info, module, permissions),
        ExecuteMsg::UpdateOperators { addrs } => execute_update_operators(deps, env, info, addrs),
        ExecuteMsg::Check { module, msg } => execute_check(deps, env, info, module, msg),
    }
}

fn execute_update_module_permissions(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: Modules,
    permissions: Vec<Permissions>,
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

    MODULE_PERMISSIONS.save(deps.storage, module.as_str(), &permissions)?;

    Ok(Response::new()
        .add_attribute("action", "execute_update_module_permissions")
        .add_attributes(
            permissions
                .iter()
                .map(|p| ("permission", p.as_str()))
                .collect::<Vec<(&str, &str)>>(),
        ))
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

fn execute_check(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    module: Modules,
    msg: Binary,
) -> Result<Response, ContractError> {
    let controller_addr = CONTROLLER_ADDR.load(deps.storage)?;

    let data: Vec<PermissionCheckMsg> = from_binary(&msg)?;
    if data.len() == 0 {
        return Err(ContractError::InvalidPermissions {});
    }

    let permissions = MODULE_PERMISSIONS.may_load(deps.storage, module.as_str())?;
    let expected_permissions = match permissions {
        Some(permissions) => permissions,
        None => return Err(ContractError::NoPermissionsForModule {}),
    };

    for permission in data {
        if !expected_permissions.contains(&permission.permission_type) {
            return Err(ContractError::InvalidPermissions {});
        }
        let _ = match permission.permission_type {
            Permissions::Ownership => {
                check_ownership_permission(&deps, &controller_addr, permission.data)
            }
            Permissions::Attribute => unimplemented!(),
        };
    }

    Ok(Response::new()
        .add_attribute("action", "execute_check_permission")
        .add_attributes(
            expected_permissions
                .iter()
                .map(|p| ("permission", p.as_str()))
                .collect::<Vec<(&str, &str)>>(),
        ))
}

fn check_ownership_permission(
    deps: &DepsMut,
    controller_addr: &Addr,
    data: Binary,
) -> Result<bool, ContractError> {
    let mint_module_addr =
        query_module_address(&deps.querier, controller_addr, Modules::MintModule)?;

    let msgs: Vec<OwnershipMsg> = from_binary(&data)?;

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
        if owner != ownership_msg.owner {
            return Err(ContractError::InvalidOwnership {});
        }
    }
    Ok(true)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ModulePermissions(module) => to_binary(&query_module_permissions(deps, module)?),
        QueryMsg::Operators {} => to_binary(&query_operators(deps)?),
    }
}

fn query_module_permissions(
    deps: Deps,
    module: Modules,
) -> StdResult<ResponseWrapper<Vec<String>>> {
    let permissions = MODULE_PERMISSIONS.load(deps.storage, module.as_str())?;
    Ok(ResponseWrapper::new(
        "module_permissions",
        permissions.iter().map(|p| p.as_str().to_string()).collect(),
    ))
}

fn query_operators(deps: Deps) -> StdResult<ResponseWrapper<Vec<String>>> {
    let addrs = OPERATORS.load(deps.storage)?;
    Ok(ResponseWrapper::new(
        "operators",
        addrs.iter().map(|a| a.to_string()).collect(),
    ))
}
