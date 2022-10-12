#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Attribute, Binary, Deps, DepsMut, Env, Event, MessageInfo, Reply, ReplyOn,
    Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use cw_utils::parse_reply_instantiate_data;

use komple_types::query::ResponseWrapper;
use komple_utils::check_admin_privileges;
use semver::Version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{
    Config, HubInfo, WebsiteConfig, CONFIG, HUB_INFO, MARBU_FEE_MODULE, MODULE_ADDRS, MODULE_ID,
    MODULE_TO_REGISTER, OPERATORS, WEBSITE_CONFIG,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:komple-hub-module";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_DESCRIPTION_LENGTH: u32 = 512;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = match msg.admin {
        Some(value) => deps.api.addr_validate(&value)?,
        None => info.sender,
    };

    let config = Config { admin };
    CONFIG.save(deps.storage, &config)?;

    if msg.hub_info.description.len() > MAX_DESCRIPTION_LENGTH as usize {
        return Err(ContractError::DescriptionTooLong {});
    }

    // Save fee module info for Marbu if exists
    // This comes from Marbu controller on Hub creation
    if let Some(marbu_fee_module) = msg.marbu_fee_module {
        let marbu_fee_module = deps.api.addr_validate(&marbu_fee_module)?;
        MARBU_FEE_MODULE.save(deps.storage, &marbu_fee_module)?;
    }

    HUB_INFO.save(deps.storage, &msg.hub_info)?;

    MODULE_ID.save(deps.storage, &0)?;

    Ok(Response::new().add_event(
        Event::new("komple_hub_module")
            .add_attribute("action", "instantiate")
            .add_attribute("admin", config.admin.to_string()),
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
            module,
            msg,
            code_id,
        } => execute_register_module(deps, env, info, module, msg, code_id),
        ExecuteMsg::UpdateHubInfo {
            name,
            description,
            image,
            external_link,
        } => execute_update_hub_info(deps, env, info, name, description, image, external_link),
        ExecuteMsg::UpdateWebsiteConfig {
            background_color,
            background_image,
            banner_image,
        } => execute_update_website_config(
            deps,
            env,
            info,
            background_color,
            background_image,
            banner_image,
        ),
        ExecuteMsg::DeregisterModule { module } => {
            execute_deregister_module(deps, env, info, module)
        }
        ExecuteMsg::UpdateOperators { addrs } => execute_update_operators(deps, env, info, addrs),
    }
}

fn execute_register_module(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: String,
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

    // Get the latest module reply id
    let module_id = (MODULE_ID.load(deps.storage)?) + 1;

    let sub_msg: SubMsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id,
            msg,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: format!("Komple Framework Module - {}", module.as_str()),
        }
        .into(),
        id: module_id,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    MODULE_ID.save(deps.storage, &module_id)?;
    // This will be loaded in reply handler for registering the correct module
    MODULE_TO_REGISTER.save(deps.storage, &module)?;

    Ok(Response::new().add_submessage(sub_msg).add_event(
        Event::new("komple_hub_module")
            .add_attribute("action", "register_module")
            .add_attribute("module", module),
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

    let mut event_attributes: Vec<Attribute> = vec![];
    if hub_info.external_link.is_some() {
        event_attributes.push(Attribute {
            key: "external_link".to_string(),
            value: hub_info.external_link.as_ref().unwrap().to_string(),
        });
    };

    Ok(Response::new().add_event(
        Event::new("komple_hub_module")
            .add_attribute("action", "update_hub_info")
            .add_attribute("name", hub_info.name)
            .add_attribute("description", hub_info.description)
            .add_attribute("image", hub_info.image)
            .add_attributes(event_attributes),
    ))
}

fn execute_update_website_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    background_color: Option<String>,
    background_image: Option<String>,
    banner_image: Option<String>,
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

    let website_config = WebsiteConfig {
        background_color,
        background_image,
        banner_image,
    };
    WEBSITE_CONFIG.save(deps.storage, &website_config)?;

    let mut event_attributes: Vec<Attribute> = vec![];
    if website_config.background_color.is_some() {
        event_attributes.push(Attribute {
            key: "background_color".to_string(),
            value: website_config
                .background_color
                .as_ref()
                .unwrap()
                .to_string(),
        });
    };
    if website_config.background_image.is_some() {
        event_attributes.push(Attribute {
            key: "background_image".to_string(),
            value: website_config
                .background_image
                .as_ref()
                .unwrap()
                .to_string(),
        });
    };
    if website_config.banner_image.is_some() {
        event_attributes.push(Attribute {
            key: "banner_image".to_string(),
            value: website_config.banner_image.as_ref().unwrap().to_string(),
        });
    };

    Ok(Response::new().add_event(
        Event::new("komple_hub_module")
            .add_attribute("action".to_string(), "update_website_config".to_string())
            .add_attributes(event_attributes),
    ))
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

    if !MODULE_ADDRS.has(deps.storage, &module) {
        return Err(ContractError::InvalidModule {});
    }

    MODULE_ADDRS.remove(deps.storage, &module);

    Ok(Response::new().add_event(
        Event::new("komple_hub_module")
            .add_attribute("action", "deregister_module")
            .add_attribute("module", module),
    ))
}

fn execute_update_operators(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut addrs: Vec<String>,
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
        Event::new("komple_hub_module")
            .add_attribute("action".to_string(), "update_operators".to_string())
            .add_attributes(event_attributes),
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::ModuleAddress { module } => to_binary(&query_module_address(deps, module)?),
        QueryMsg::Operators {} => to_binary(&query_operators(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ResponseWrapper<ConfigResponse>> {
    let config = CONFIG.load(deps.storage)?;
    let hub_info = HUB_INFO.load(deps.storage)?;
    let website_config = WEBSITE_CONFIG.may_load(deps.storage)?;
    Ok(ResponseWrapper::new(
        "config",
        ConfigResponse {
            admin: config.admin.to_string(),
            hub_info,
            website_config,
        },
    ))
}

fn query_module_address(deps: Deps, module: String) -> StdResult<ResponseWrapper<String>> {
    let addr = MODULE_ADDRS.load(deps.storage, module.as_str())?;
    Ok(ResponseWrapper::new("module_addrSess", addr.to_string()))
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
    let module_id = MODULE_ID.load(deps.storage)?;

    // Check if the reply id is the same
    if msg.id != module_id {
        return Err(ContractError::InvalidReplyID {});
    };

    // Get the module for registering
    let module_to_register = MODULE_TO_REGISTER.load(deps.storage)?;

    // Handle the registration
    handle_module_instantiate_reply(deps, msg, module_to_register.as_str())
}

fn handle_module_instantiate_reply(
    deps: DepsMut,
    msg: Reply,
    module_to_register: &str,
) -> Result<Response, ContractError> {
    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            MODULE_ADDRS.save(
                deps.storage,
                module_to_register,
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
