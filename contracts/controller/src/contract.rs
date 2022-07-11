#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Reply, ReplyOn,
    Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, ModuleAddrResponse, QueryMsg};
use crate::state::{Config, ControllerInfo, CONFIG, CONTROLLER_INFO, MINT_MODULE_ADDR};

use mint::msg::InstantiateMsg as MintModuleInstantiateMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:controller";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MINT_MODULE_INSTANTIATE_REPLY_ID: u64 = 1;
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
    }
}

fn execute_init_mint_module(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    code_id: u64,
) -> Result<Response, ContractError> {
    can_execute(&deps, &info)?;

    let config = CONFIG.load(deps.storage)?;

    let msg: SubMsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id,
            msg: to_binary(&MintModuleInstantiateMsg {
                admin: config.admin.to_string(),
            })?,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: String::from("Framework mint contract"),
        }
        .into(),
        id: MINT_MODULE_INSTANTIATE_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    Ok(Response::new()
        .add_submessage(msg)
        .add_attribute("action", "init_mint_module"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::ContollerInfo {} => unimplemented!(),
        QueryMsg::MintModuleAddr {} => to_binary(&query_mint_module_addr(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

fn query_mint_module_addr(deps: Deps) -> StdResult<ModuleAddrResponse> {
    let addr = MINT_MODULE_ADDR.load(deps.storage)?;
    Ok(ModuleAddrResponse {
        addr: addr.to_string(),
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        MINT_MODULE_INSTANTIATE_REPLY_ID => handle_mint_module_instantiate_reply(deps, msg),
        _ => return Err(ContractError::InvalidReplyID {}),
    }
}

fn handle_mint_module_instantiate_reply(
    deps: DepsMut,
    msg: Reply,
) -> Result<Response, ContractError> {
    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            MINT_MODULE_ADDR.save(deps.storage, &Addr::unchecked(res.contract_address))?;
            Ok(Response::default().add_attribute("action", "instantiate_mint_module_reply"))
        }
        Err(_) => Err(ContractError::MintInstantiateError {}),
    }
}

fn can_execute(deps: &DepsMut, info: &MessageInfo) -> Result<bool, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    Ok(true)
}
