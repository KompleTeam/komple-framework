#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use cw721::OwnerOfResponse;
use rift_types::collection::Collections;
use rift_types::module::Modules;
use rift_types::permission::Permissions;
use rift_types::query::MultipleAddressResponse;
use rift_utils::{check_admin_privileges, get_collection_address, get_module_address};

use token_contract::msg::QueryMsg as TokenQueryMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, OwnershipMsg, PermissionCheckMsg, QueryMsg};
use crate::state::{Config, CONFIG, CONTROLLER_ADDR, MODULE_PERMISSIONS, WHITELIST_ADDRS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:permission-module";
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
        ExecuteMsg::UpdateWhitelistAddresses { addrs } => {
            execute_update_whitelist_addresses(deps, env, info, addrs)
        }
        ExecuteMsg::Check { module, msg } => execute_check(deps, env, info, module, msg),
    }
}

fn execute_update_module_permissions(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    module: Modules,
    permissions: Vec<Permissions>,
) -> Result<Response, ContractError> {
    let controller_addr = CONTROLLER_ADDR.load(deps.storage)?;
    let whitelist_addrs = WHITELIST_ADDRS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &config.admin,
        Some(&controller_addr),
        whitelist_addrs,
    )?;

    MODULE_PERMISSIONS.save(deps.storage, module.to_string(), &permissions)?;

    Ok(Response::new()
        .add_attribute("action", "execute_update_module_permissions")
        .add_attributes(
            permissions
                .iter()
                .map(|p| ("permission", p.to_string()))
                .collect::<Vec<(&str, &str)>>(),
        ))
}

fn execute_update_whitelist_addresses(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    addrs: Vec<String>,
) -> Result<Response, ContractError> {
    let controller_addr = CONTROLLER_ADDR.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(&info.sender, &config.admin, Some(&controller_addr), None)?;

    let whitelist_addrs = addrs
        .iter()
        .map(|addr| -> StdResult<Addr> {
            let addr = deps.api.addr_validate(addr)?;
            Ok(addr)
        })
        .collect::<StdResult<Vec<Addr>>>()?;

    WHITELIST_ADDRS.save(deps.storage, &whitelist_addrs)?;

    Ok(Response::new().add_attribute("action", "execute_update_whitelist_addresses"))
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

    let expected_permissions = MODULE_PERMISSIONS.load(deps.storage, module.to_string())?;

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
                .map(|p| ("permission", p.to_string()))
                .collect::<Vec<(&str, &str)>>(),
        ))
}

fn check_ownership_permission(
    deps: &DepsMut,
    controller_addr: &Addr,
    data: Binary,
) -> Result<bool, ContractError> {
    let mint_module_address = get_module_address(deps, controller_addr, Modules::MintModule)?;
    let passcard_module_address =
        get_module_address(deps, controller_addr, Modules::PasscardModule)?;

    let msgs: Vec<OwnershipMsg> = from_binary(&data)?;

    for ownership_msg in msgs {
        let address: Addr;
        match ownership_msg.collection_type {
            // TODO: Could implement a map for easy lookup of collection address
            Collections::Normal => {
                address = get_collection_address(
                    &deps,
                    &mint_module_address,
                    ownership_msg.collection_id,
                )?;
            }
            Collections::Passcard => {
                address = get_collection_address(
                    &deps,
                    &passcard_module_address,
                    ownership_msg.collection_id,
                )?;
            }
        }
        let msg = TokenQueryMsg::OwnerOf {
            token_id: ownership_msg.token_id.to_string(),
            include_expired: None,
        };
        let res: OwnerOfResponse = deps.querier.query_wasm_smart(address, &msg)?;
        if res.owner != ownership_msg.owner {
            return Err(ContractError::InvalidOwnership {});
        }
    }
    Ok(true)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ModulePermissions(module) => to_binary(&query_module_permissions(deps, module)?),
        QueryMsg::WhitelistAddresses {} => to_binary(&query_whitelist_addresses(deps)?),
    }
}

fn query_module_permissions(deps: Deps, module: Modules) -> StdResult<Vec<Permissions>> {
    let permissions = MODULE_PERMISSIONS.load(deps.storage, module.to_string())?;
    Ok(permissions)
}

fn query_whitelist_addresses(deps: Deps) -> StdResult<MultipleAddressResponse> {
    let addrs = WHITELIST_ADDRS.load(deps.storage)?;
    Ok(MultipleAddressResponse {
        addresses: addrs.iter().map(|a| a.to_string()).collect(),
    })
}
