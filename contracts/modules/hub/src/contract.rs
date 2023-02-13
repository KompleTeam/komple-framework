#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Reply, ReplyOn,
    Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use cw_storage_plus::Bound;
use cw_utils::{parse_reply_instantiate_data};

use komple_framework_types::shared::execute::SharedExecuteMsg;
use komple_framework_types::shared::query::ResponseWrapper;
use komple_framework_types::shared::RegisterMsg;
use komple_framework_utils::check_admin_privileges;
use komple_framework_utils::response::{EventHelper, ResponseHelper};
use komple_framework_utils::shared::execute_update_operators;
use semver::Version;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, ModulesResponse, QueryMsg,
};
use crate::state::{
    Config, HubInfo, CONFIG, HUB_INFO, MARBU_FEE_MODULE, MODULES, MODULE_ID, MODULE_TO_REGISTER,
    OPERATORS,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:komple-framework-hub-module";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_DESCRIPTION_LENGTH: u32 = 512;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: RegisterMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Return error if instantiate data is not sent
    if msg.data.is_none() {
        return Err(ContractError::InvalidInstantiateMsg {});
    };
    let data: InstantiateMsg = from_binary(&msg.data.unwrap())?;

    let admin = deps.api.addr_validate(&msg.admin)?;
    let config = Config { admin };
    CONFIG.save(deps.storage, &config)?;

    if data.hub_info.description.len() > MAX_DESCRIPTION_LENGTH as usize {
        return Err(ContractError::DescriptionTooLong {});
    }

    // Save fee module info for Marbu if exists
    // This comes from Marbu Controller on Hub creation
    if let Some(marbu_fee_module) = data.marbu_fee_module {
        let marbu_fee_module = deps.api.addr_validate(&marbu_fee_module)?;
        MARBU_FEE_MODULE.save(deps.storage, &marbu_fee_module)?;
    }

    HUB_INFO.save(deps.storage, &data.hub_info)?;

    MODULE_ID.save(deps.storage, &0)?;

    Ok(ResponseHelper::new_module("hub", "instantiate").add_event(
        EventHelper::new("hub_instantiate")
            .add_attribute("admin", config.admin)
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
        ExecuteMsg::RegisterModule {
            code_id,
            module,
            msg,
        } => execute_register_module(deps, env, info, code_id, module, msg),
        ExecuteMsg::UpdateHubInfo {
            name,
            description,
            image,
            external_link,
        } => execute_update_hub_info(deps, env, info, name, description, image, external_link),
        ExecuteMsg::DeregisterModule { module } => {
            execute_deregister_module(deps, env, info, module)
        }
        ExecuteMsg::UpdateOperators { addrs } => {
            let config = CONFIG.load(deps.storage)?;
            let res = execute_update_operators(
                deps,
                info,
                "hub",
                &env.contract.address,
                &config.admin,
                OPERATORS,
                addrs,
            );
            match res {
                Ok(res) => Ok(res),
                Err(err) => Err(err.into()),
            }
        }
        ExecuteMsg::MigrateContracts {
            code_id,
            contract_address,
            msg,
        } => execute_migrate_contracts(deps, env, info, code_id, contract_address, msg),
    }
}

fn execute_register_module(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    code_id: u64,
    module: String,
    msg: Option<Binary>,
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

    // Get the latest module reply id
    let module_id = (MODULE_ID.load(deps.storage)?) + 1;

    // Register message to instantiate the module
    // Admin is set as the hub's admin
    // Additional data is sent to the module
    let register_msg = RegisterMsg {
        admin: config.admin.to_string(),
        data: msg,
    };

    let sub_msg: SubMsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id,
            msg: to_binary(&register_msg)?,
            funds: vec![],
            admin: Some(env.contract.address.to_string()),
            label: format!("Komple Framework Module - {}", module.as_str()),
        }
        .into(),
        id: module_id,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    MODULE_ID.save(deps.storage, &module_id)?;
    // Module name will be loaded in reply handler for saving
    // the correct module name to storage
    MODULE_TO_REGISTER.save(deps.storage, &module)?;

    Ok(ResponseHelper::new_module("hub", "register_module")
        .add_submessage(sub_msg)
        .add_event(
            EventHelper::new("hub_register_module")
                .add_attribute("module", module)
                .get(),
        ))
}

fn execute_update_hub_info(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    description: String,
    image: String,
    external_link: Option<String>,
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

    let hub_info = HubInfo {
        name,
        description,
        image,
        external_link,
    };
    HUB_INFO.save(deps.storage, &hub_info)?;

    Ok(
        ResponseHelper::new_module("hub", "update_hub_info").add_event(
            EventHelper::new("hub_update_hub_info")
                .add_attribute("name", hub_info.name)
                .add_attribute("description", hub_info.description)
                .add_attribute("image", hub_info.image)
                .check_add_attribute(
                    &hub_info.external_link,
                    "external_link",
                    hub_info.external_link.as_ref().unwrap_or(&String::from("")),
                )
                .get(),
        ),
    )
}

fn execute_deregister_module(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: String,
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

    let module_addr = MODULES.load(deps.storage, module.clone());
    if module_addr.is_err() {
        return Err(ContractError::InvalidModule {});
    }

    let mut msgs: Vec<WasmMsg> = vec![];

    // Create a message to disable execute messages on module
    msgs.push(WasmMsg::Execute {
        contract_addr: module_addr.as_ref().unwrap().to_string(),
        msg: to_binary(&SharedExecuteMsg::LockExecute {})?,
        funds: vec![],
    });

    // Create a message to set contract's admin as None
    msgs.push(WasmMsg::ClearAdmin {
        contract_addr: module_addr.unwrap().to_string(),
    });

    MODULES.remove(deps.storage, module.clone());

    Ok(ResponseHelper::new_module("hub", "deregister_module")
        .add_messages(msgs)
        .add_event(
            EventHelper::new("hub_deregister_module")
                .add_attribute("module", module)
                .get(),
        ))
}

fn execute_migrate_contracts(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    code_id: u64,
    contract_address: String,
    msg: Binary,
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

    let contract_addr = deps.api.addr_validate(&contract_address)?;

    let msg = WasmMsg::Migrate {
        contract_addr: contract_addr.to_string(),
        new_code_id: code_id,
        msg,
    };

    Ok(ResponseHelper::new_module("hub", "migrate_contracts")
        .add_message(msg)
        .add_event(
            EventHelper::new("hub_migrate_contracts")
                .add_attribute("code_id", code_id.to_string())
                .add_attribute("contract_address", contract_address)
                .get(),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::ModuleAddress { module } => to_binary(&query_module_address(deps, module)?),
        QueryMsg::Modules { start_after, limit } => {
            to_binary(&query_modules(deps, start_after, limit)?)
        }
        QueryMsg::Operators {} => to_binary(&query_operators(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ResponseWrapper<ConfigResponse>> {
    let config = CONFIG.load(deps.storage)?;
    let hub_info = HUB_INFO.load(deps.storage)?;
    Ok(ResponseWrapper::new(
        "config",
        ConfigResponse {
            admin: config.admin.to_string(),
            hub_info,
        },
    ))
}

fn query_module_address(deps: Deps, module: String) -> StdResult<ResponseWrapper<String>> {
    let addr = MODULES.load(deps.storage, module.to_string())?;
    Ok(ResponseWrapper::new("module_address", addr.to_string()))
}

fn query_modules(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u8>,
) -> StdResult<ResponseWrapper<Vec<ModulesResponse>>> {
    let limit = limit.unwrap_or(10) as usize;
    let start = start_after.map(Bound::exclusive);

    let modules = MODULES
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (name, address) = item.unwrap();
            ModulesResponse {
                name,
                address: address.to_string(),
            }
        })
        .collect::<Vec<ModulesResponse>>();

    Ok(ResponseWrapper::new("modules", modules))
}

fn query_operators(deps: Deps) -> StdResult<ResponseWrapper<Vec<String>>> {
    let addrs = OPERATORS.may_load(deps.storage)?;
    let addrs = match addrs {
        Some(addrs) => addrs.iter().map(|a| a.to_string()).collect(),
        None => vec![],
    };
    Ok(ResponseWrapper::new("operators", addrs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    // Get the last module id
    // This is used as the reply id
    let module_id = MODULE_ID.load(deps.storage)?;

    // Check if the reply id is the same
    if msg.id != module_id {
        return Err(ContractError::InvalidReplyID {});
    };

    // Handle the registration
    handle_module_instantiate_reply(deps, msg)
}

fn handle_module_instantiate_reply(deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    let reply = parse_reply_instantiate_data(msg);

    // Get the module for registering
    let module_to_register = MODULE_TO_REGISTER.load(deps.storage)?;

    match reply {
        Ok(res) => {
            MODULES.save(
                deps.storage,
                module_to_register.clone(),
                &Addr::unchecked(res.contract_address),
            )?;
            Ok(Response::default().add_attribute(
                "action",
                format!("instantiate_{}_module_reply", module_to_register),
            ))
        }
        Err(_) => Err(ContractError::ModuleInstantiateError {
            module: module_to_register.to_string(),
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
