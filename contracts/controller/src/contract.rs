#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, ControllerInfo, COLLECTION_ID, CONFIG, CONTROLLER_INFO};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MINT_INSTANTIATE_REPLY_ID: u64 = 1;
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
        mint_code_id: msg.mint_code_id,
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

    COLLECTION_ID.save(deps.storage, &0)?;

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
        ExecuteMsg::UpdateMintCodeId { code_id } => {
            execute_update_mint_code_id(deps, env, info, code_id)
        }
        ExecuteMsg::AddCollection {
            mint_code_id,
            mint_instantiate_msg,
        } => unimplemented!(),
    }
}

fn execute_update_mint_code_id(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    code_id: u64,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    };

    if code_id == 0 {
        return Err(ContractError::InvalidCodeId {});
    }

    config.mint_code_id = code_id;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_mint_code_id")
        .add_attribute("code_id", code_id.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::GetCollection { collection_id } => unimplemented!(),
        QueryMsg::GetContollerInfo {} => unimplemented!(),
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}
