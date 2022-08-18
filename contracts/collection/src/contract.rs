#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError,
    StdResult, SubMsg, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use cw_utils::parse_reply_instantiate_data;

use komple_types::module::{
    Modules, MARKETPLACE_MODULE_INSTANTIATE_REPLY_ID, MERGE_MODULE_INSTANTIATE_REPLY_ID,
    MINT_MODULE_INSTANTIATE_REPLY_ID, PERMISSION_MODULE_INSTANTIATE_REPLY_ID,
};
use komple_types::{
    instantiate::{MarketplaceModuleInstantiateMsg, ModuleInstantiateMsg},
    query::ResponseWrapper,
};
use komple_utils::check_admin_privileges;
use semver::Version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{
    Config, CollectionInfo, WebsiteConfig, CONFIG, COLLECTION_INFO, MODULE_ADDR, WEBSITE_CONFIG,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:komple-collection-contract";
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

    let config = Config {
        admin: info.sender.clone(),
    };
    CONFIG.save(deps.storage, &config)?;

    if msg.description.len() > MAX_DESCRIPTION_LENGTH as usize {
        return Err(ContractError::DescriptionTooLong {});
    }

    let collection_info = CollectionInfo {
        name: msg.name,
        description: msg.description,
        image: msg.image,
        external_link: msg.external_link,
    };
    COLLECTION_INFO.save(deps.storage, &collection_info)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::InitMintModule { code_id } => {
            execute_init_mint_module(deps, env, info, code_id)
        }
        ExecuteMsg::InitPermissionModule { code_id } => {
            execute_init_permission_module(deps, env, info, code_id)
        }
        ExecuteMsg::InitMergeModule { code_id } => {
            execute_init_merge_module(deps, env, info, code_id)
        }
        ExecuteMsg::InitMarketplaceModule {
            code_id,
            native_denom,
        } => execute_init_marketplace_module(deps, env, info, code_id, native_denom),
        ExecuteMsg::UpdateCollectionInfo {
            name,
            description,
            image,
            external_link,
        } => {
            execute_update_collection_info(deps, env, info, name, description, image, external_link)
        }
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
    }
}

fn execute_init_mint_module(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    code_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        None,
        None,
    )?;

    let msg: SubMsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id,
            msg: to_binary(&ModuleInstantiateMsg {
                admin: config.admin.to_string(),
            })?,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: String::from("Komple Framework mint module"),
        }
        .into(),
        id: MINT_MODULE_INSTANTIATE_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    Ok(Response::new()
        .add_submessage(msg)
        .add_attribute("action", "execute_init_mint_module"))
}

fn execute_init_permission_module(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    code_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        None,
        None,
    )?;

    let msg: SubMsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id,
            msg: to_binary(&ModuleInstantiateMsg {
                admin: config.admin.to_string(),
            })?,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: String::from("Komple Framework permission module"),
        }
        .into(),
        id: PERMISSION_MODULE_INSTANTIATE_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    Ok(Response::new()
        .add_submessage(msg)
        .add_attribute("action", "execute_init_permission_module"))
}

fn execute_init_merge_module(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    code_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        None,
        None,
    )?;

    let msg: SubMsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id,
            msg: to_binary(&ModuleInstantiateMsg {
                admin: config.admin.to_string(),
            })?,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: String::from("Komple Framework merge module"),
        }
        .into(),
        id: MERGE_MODULE_INSTANTIATE_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    Ok(Response::new()
        .add_submessage(msg)
        .add_attribute("action", "execute_init_merge_module"))
}

fn execute_init_marketplace_module(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    code_id: u64,
    native_denom: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        None,
        None,
    )?;

    let msg: SubMsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id,
            msg: to_binary(&MarketplaceModuleInstantiateMsg {
                admin: config.admin.to_string(),
                native_denom,
            })?,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: String::from("Komple Framework Marketplace Module"),
        }
        .into(),
        id: MARKETPLACE_MODULE_INSTANTIATE_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    Ok(Response::new()
        .add_submessage(msg)
        .add_attribute("action", "execute_init_marketplace_module"))
}

fn execute_update_collection_info(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    description: String,
    image: String,
    external_link: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        None,
        None,
    )?;

    let collection_info = CollectionInfo {
        name,
        description,
        image,
        external_link,
    };
    COLLECTION_INFO.save(deps.storage, &collection_info)?;

    Ok(Response::new().add_attribute("action", "execute_update_collection_info"))
}

fn execute_update_website_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    background_color: Option<String>,
    background_image: Option<String>,
    banner_image: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        None,
        None,
    )?;

    let website_config = WebsiteConfig {
        background_color,
        background_image,
        banner_image,
    };
    WEBSITE_CONFIG.save(deps.storage, &website_config)?;

    Ok(Response::new().add_attribute("action", "execute_update_website_config"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::ModuleAddress(module) => to_binary(&query_module_address(deps, module)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ResponseWrapper<ConfigResponse>> {
    let config = CONFIG.load(deps.storage)?;
    let collection_info = COLLECTION_INFO.load(deps.storage)?;
    let website_config = WEBSITE_CONFIG.may_load(deps.storage)?;
    Ok(ResponseWrapper::new(
        "config",
        ConfigResponse {
            admin: config.admin.to_string(),
            collection_info,
            website_config,
        },
    ))
}

fn query_module_address(deps: Deps, module: Modules) -> StdResult<ResponseWrapper<String>> {
    let addr = MODULE_ADDR.load(deps.storage, module.as_str())?;
    Ok(ResponseWrapper::new("module_address", addr.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        MINT_MODULE_INSTANTIATE_REPLY_ID => {
            handle_module_instantiate_reply(deps, msg, Modules::MintModule)
        }
        PERMISSION_MODULE_INSTANTIATE_REPLY_ID => {
            handle_module_instantiate_reply(deps, msg, Modules::PermissionModule)
        }
        MERGE_MODULE_INSTANTIATE_REPLY_ID => {
            handle_module_instantiate_reply(deps, msg, Modules::MergeModule)
        }
        MARKETPLACE_MODULE_INSTANTIATE_REPLY_ID => {
            handle_module_instantiate_reply(deps, msg, Modules::MarketplaceModule)
        }
        _ => return Err(ContractError::InvalidReplyID {}),
    }
}

fn handle_module_instantiate_reply(
    deps: DepsMut,
    msg: Reply,
    module: Modules,
) -> Result<Response, ContractError> {
    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            MODULE_ADDR.save(
                deps.storage,
                module.as_str(),
                &Addr::unchecked(res.contract_address),
            )?;
            Ok(Response::default().add_attribute(
                "action",
                format!("instantiate_{}_module_reply", module.as_str()),
            ))
        }
        Err(_) => Err(ContractError::ModuleInstantiateError {
            module: module.as_str().to_string(),
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
