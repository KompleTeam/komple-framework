#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdResult,
    SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

use rift_types::module::{
    Modules, MINT_MODULE_INSTANTIATE_REPLY_ID, PERMISSION_MODULE_INSTANTIATE_REPLY_ID,
};
use rift_types::query::AddressResponse;

use mint_module::msg::InstantiateMsg as MintModuleInstantiateMsg;

use permission_module::msg::InstantiateMsg as PermissionModuleInstantiateMsg;
use rift_utils::have_admin_privilages;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, ControllerInfo, CONFIG, CONTROLLER_INFO, MODULE_ADDR};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:controller-contract";
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

    let controller_info = ControllerInfo {
        name: msg.name,
        description: msg.description,
        image: msg.image,
        external_link: msg.external_link,
    };
    CONTROLLER_INFO.save(deps.storage, &controller_info)?;

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
    }
}

fn execute_init_mint_module(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    code_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if !have_admin_privilages(&info.sender, &config.admin, None, None) {
        return Err(ContractError::Unauthorized {});
    }

    let msg: SubMsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id,
            msg: to_binary(&MintModuleInstantiateMsg {
                admin: config.admin.to_string(),
            })?,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: String::from("Framework mint module"),
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
    _env: Env,
    info: MessageInfo,
    code_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if !have_admin_privilages(&info.sender, &config.admin, None, None) {
        return Err(ContractError::Unauthorized {});
    }

    let msg: SubMsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id,
            msg: to_binary(&PermissionModuleInstantiateMsg {
                admin: config.admin.to_string(),
            })?,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: String::from("Framework permission module"),
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::ContollerInfo {} => unimplemented!(),
        QueryMsg::ModuleAddress(module) => to_binary(&query_module_address(deps, module)?),
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

fn query_module_address(deps: Deps, module: Modules) -> StdResult<AddressResponse> {
    let addr = MODULE_ADDR.load(deps.storage, module.to_string())?;
    Ok(AddressResponse {
        address: addr.to_string(),
    })
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
                module.to_string(),
                &Addr::unchecked(res.contract_address),
            )?;
            Ok(Response::default().add_attribute(
                "action",
                format!("instantiate_{}_module_reply", module.to_string()),
            ))
        }
        Err(_) => Err(ContractError::ModuleInstantiateError {
            module: module.to_string().to_string(),
        }),
    }
}
