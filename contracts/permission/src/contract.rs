#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Attribute, Binary, CosmosMsg, Deps, DepsMut, Empty, Env,
    MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use cw_utils::parse_reply_instantiate_data;
use komple_types::module::Modules;
use komple_types::query::ResponseWrapper;
use komple_utils::check_admin_privileges;
use komple_utils::event::EventHelper;
use semver::Version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, PermissionCheckMsg, QueryMsg};
use crate::state::{
    Config, CONFIG, HUB_ADDR, MODULE_PERMISSIONS, OPERATORS, PERMISSIONS, PERMISSION_ID,
    PERMISSION_TO_REGISTER,
};

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

    HUB_ADDR.save(deps.storage, &info.sender)?;

    PERMISSION_ID.save(deps.storage, &0)?;

    Ok(Response::new().add_event(
        EventHelper::new("komple_permission_module")
            .add_attribute("action", "instantiate")
            .add_attribute("admin", config.admin.to_string())
            .add_attribute("hub_addr", info.sender.to_string())
            .get(),
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterPermission {
            permission,
            msg,
            code_id,
        } => execute_register_permission(deps, env, info, permission, msg, code_id),
        ExecuteMsg::UpdateModulePermissions {
            module,
            permissions,
        } => execute_update_module_permissions(deps, env, info, module, permissions),
        ExecuteMsg::UpdateOperators { addrs } => execute_update_operators(deps, env, info, addrs),
        ExecuteMsg::Check { module, msg } => execute_check(deps, env, info, module, msg),
    }
}

fn execute_register_permission(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    permission: String,
    msg: Binary,
    code_id: u64,
) -> Result<Response, ContractError> {
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        None,
        operators,
    )?;

    // Get the latest permission reply id
    let permission_id = (PERMISSION_ID.load(deps.storage)?) + 1;

    let sub_msg: SubMsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id,
            msg,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: format!("Komple Permission Module - {}", permission.as_str()),
        }
        .into(),
        id: permission_id,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    PERMISSION_ID.save(deps.storage, &permission_id)?;
    // This will be loaded in reply handler for registering the correct module
    PERMISSION_TO_REGISTER.save(deps.storage, &permission)?;

    Ok(Response::new().add_submessage(sub_msg).add_event(
        EventHelper::new("komple_permission_module")
            .add_attribute("action", "register_permission")
            .add_attribute("module", permission.as_str())
            .get(),
    ))
}

fn execute_update_module_permissions(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: String,
    permissions: Vec<String>,
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

    let mut event_attributes: Vec<Attribute> = vec![];

    for permission in &permissions {
        if !PERMISSIONS.has(deps.storage, permission) {
            return Err(ContractError::InvalidPermissions {});
        };
        event_attributes.push(Attribute {
            key: "permission".to_string(),
            value: permission.to_string(),
        });
    }

    MODULE_PERMISSIONS.save(deps.storage, &module, &permissions)?;

    Ok(Response::new().add_event(
        EventHelper::new("komple_permission_module")
            .add_attribute("action", "update_module_permissions")
            .add_attribute("module", module)
            .add_attributes(event_attributes)
            .get(),
    ))
}

fn execute_update_operators(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut addrs: Vec<String>,
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

    addrs.sort_unstable();
    addrs.dedup();

    let mut event_attributes: Vec<Attribute> = vec![];

    let addrs = addrs
        .iter()
        .map(|addr| -> StdResult<Addr> {
            let addr = deps.api.addr_validate(addr)?;
            event_attributes.push(Attribute {
                key: "addrs".to_string(),
                value: addr.to_string(),
            });
            Ok(addr)
        })
        .collect::<StdResult<Vec<Addr>>>()?;

    OPERATORS.save(deps.storage, &addrs)?;

    Ok(Response::new().add_event(
        EventHelper::new("komple_permission_module")
            .add_attribute("action".to_string(), "update_operators".to_string())
            .add_attributes(event_attributes)
            .get(),
    ))
}

fn execute_check(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    module: String,
    msg: Binary,
) -> Result<Response, ContractError> {
    let mut msgs: Vec<CosmosMsg> = vec![];

    let data: Vec<PermissionCheckMsg> = from_binary(&msg)?;
    if data.is_empty() {
        return Err(ContractError::InvalidPermissions {});
    }

    let permissions = MODULE_PERMISSIONS.may_load(deps.storage, module.as_str())?;
    let expected_permissions = match permissions {
        Some(permissions) => permissions,
        None => return Err(ContractError::NoPermissionsForModule {}),
    };

    let mut event_attributes: Vec<Attribute> = vec![];

    for permission in data {
        if !expected_permissions.contains(&permission.permission_type) {
            return Err(ContractError::InvalidPermissions {});
        }
        let addr = PERMISSIONS.load(deps.storage, &permission.permission_type)?;
        let permission_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: addr.to_string(),
            msg: permission.data,
            funds: vec![],
        });
        msgs.push(permission_msg);

        event_attributes.push(Attribute {
            key: "permission".to_string(),
            value: permission.permission_type,
        });
    }

    Ok(Response::new().add_event(
        EventHelper::new("komple_permission_module")
            .add_attribute("action", "check")
            .add_attribute("module", module)
            .add_attributes(event_attributes)
            .get(),
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::PermissionAddress { permission } => {
            to_binary(&query_permission_address(deps, permission)?)
        }
        QueryMsg::ModulePermissions(module) => to_binary(&query_module_permissions(deps, module)?),
        QueryMsg::Operators {} => to_binary(&query_operators(deps)?),
    }
}

fn query_permission_address(deps: Deps, permission: String) -> StdResult<ResponseWrapper<String>> {
    let addr = PERMISSIONS.load(deps.storage, &permission)?;
    Ok(ResponseWrapper::new("permission_address", addr.to_string()))
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    // Get the last permission id
    let permission_id = PERMISSION_ID.load(deps.storage)?;

    // Check if the reply id is the same
    if msg.id != permission_id {
        return Err(ContractError::InvalidReplyID {});
    };

    // Get the module for registering
    let permission_to_register = PERMISSION_TO_REGISTER.load(deps.storage)?;

    // Handle the registration
    handle_permission_instantiate_reply(deps, msg, permission_to_register.as_str())
}

fn handle_permission_instantiate_reply(
    deps: DepsMut,
    msg: Reply,
    permission_to_register: &str,
) -> Result<Response, ContractError> {
    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            PERMISSIONS.save(
                deps.storage,
                permission_to_register,
                &Addr::unchecked(res.contract_address),
            )?;
            Ok(Response::default().add_attribute(
                "action",
                format!("instantiate_{}_permission_reply", permission_to_register),
            ))
        }
        Err(_) => Err(ContractError::PermissionInstantiateError {
            permission: permission_to_register.to_string(),
        }),
    }
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
