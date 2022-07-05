#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdResult,
    SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetCollectionResponse, InstantiateMsg, QueryMsg};
use crate::state::{Config, ControllerInfo, COLLECTIONS, COLLECTION_ID, CONFIG, CONTROLLER_INFO};

use mint::msg::InstantiateMsg as MintInstantiateMsg;

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
        ExecuteMsg::AddCollection { instantiate_msg } => {
            execute_add_collection(deps, env, info, instantiate_msg)
        }
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

fn execute_add_collection(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    instantiate_msg: MintInstantiateMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    };

    let collection_id = (COLLECTION_ID.load(deps.storage)?) + 1;

    let msg: SubMsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id: config.mint_code_id,
            msg: to_binary(&MintInstantiateMsg {
                symbol: instantiate_msg.symbol,
                collection_info: instantiate_msg.collection_info.clone(),
                per_address_limit: instantiate_msg.per_address_limit,
                whitelist: instantiate_msg.whitelist,
                start_time: instantiate_msg.start_time,
                token_code_id: instantiate_msg.token_code_id,
            })?,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: String::from("Framework mint contract"),
        }
        .into(),
        id: MINT_INSTANTIATE_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    COLLECTION_ID.save(deps.storage, &collection_id)?;

    Ok(Response::new()
        .add_submessage(msg)
        .add_attribute("action", "add_collection")
        .add_attribute("collection_id", collection_id.to_string())
        .add_attribute("name", instantiate_msg.collection_info.name))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::GetCollection { collection_id } => {
            to_binary(&query_get_collection(deps, collection_id)?)
        }
        QueryMsg::GetContollerInfo {} => unimplemented!(),
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

fn query_get_collection(deps: Deps, collection_id: u32) -> StdResult<GetCollectionResponse> {
    let address = COLLECTIONS.load(deps.storage, collection_id)?;
    Ok(GetCollectionResponse {
        address: address.to_string(),
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != MINT_INSTANTIATE_REPLY_ID {
        return Err(ContractError::InvalidReplyID {});
    }

    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            let collection_id = COLLECTION_ID.load(deps.storage)?;
            COLLECTIONS.save(
                deps.storage,
                collection_id,
                &Addr::unchecked(res.contract_address),
            )?;
            Ok(Response::default().add_attribute("action", "instantiate_mint_reply"))
        }
        Err(_) => Err(ContractError::MintInstantiateError {}),
    }
}
