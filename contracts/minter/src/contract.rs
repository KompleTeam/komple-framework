#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Reply, ReplyOn,
    Response, StdResult, SubMsg, Timestamp, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;
use token::state::Locks;
use url::Url;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TokenAddressResponse};
use crate::state::{CollectionInfo, Config, COLLECTION_INFO, CONFIG, TOKEN_ADDR, TOKEN_ID};

use cw721_base::{InstantiateMsg as TokenInstantiateMsg, MintMsg};
use token::msg::ExecuteMsg as TokenExecuteMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const TOKEN_INSTANTIATE_REPLY_ID: u64 = 1;
const MAX_DESCRIPTION_LENGTH: u32 = 512;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let whitelist = msg
        .whitelist
        .and_then(|w| deps.api.addr_validate(w.as_str()).ok());

    if msg.start_time.is_some() && env.block.time > msg.start_time.unwrap() {
        return Err(ContractError::InvalidStartTime {});
    }

    let config = Config {
        admin: info.sender.clone(),
        // TODO: Implement royalty
        royalty_info: None,
        per_address_limit: msg.per_address_limit,
        whitelist,
        start_time: msg.start_time,
        mint_lock: false,
    };
    CONFIG.save(deps.storage, &config)?;

    if msg.collection_info.description.len() > MAX_DESCRIPTION_LENGTH as usize {
        return Err(ContractError::DescriptionTooLong {});
    }

    Url::parse(&msg.collection_info.image)?;

    if let Some(ref external_link) = msg.collection_info.external_link {
        Url::parse(external_link)?;
    }

    let collection_info = CollectionInfo {
        name: msg.collection_info.name.clone(),
        description: msg.collection_info.description,
        image: msg.collection_info.image,
        external_link: msg.collection_info.external_link,
    };
    COLLECTION_INFO.save(deps.storage, &collection_info)?;

    TOKEN_ID.save(deps.storage, &0)?;

    // Instantiate token contract
    let sub_msg: SubMsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id: msg.token_code_id,
            msg: to_binary(&TokenInstantiateMsg {
                name: msg.collection_info.name.clone(),
                symbol: msg.symbol,
                minter: env.contract.address.to_string(),
            })?,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: String::from("Framework wrapped token"),
        }
        .into(),
        id: TOKEN_INSTANTIATE_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("minter", env.contract.address)
        .add_attribute("collection_name", msg.collection_info.name)
        .add_submessage(sub_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateLocks { locks } => execute_update_locks(deps, env, info, locks),
        ExecuteMsg::Mint { recipient } => execute_mint(deps, env, info, recipient),
        ExecuteMsg::SetWhitelist { whitelist } => execute_set_whitelist(deps, env, info, whitelist),
        ExecuteMsg::UpdateStartTime(start_time) => {
            execute_update_start_time(deps, env, info, start_time)
        }
        ExecuteMsg::UpdatePerAddressLimit { per_address_limit } => {
            execute_update_per_address_limit(deps, env, info, per_address_limit)
        }
    }
}

pub fn execute_update_locks(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    locks: Locks,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let token_address = TOKEN_ADDR.load(deps.storage)?;

    let update_lock_msg: TokenExecuteMsg<Empty> = TokenExecuteMsg::UpdateLocks {
        locks: locks.clone(),
    };
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_address.to_string(),
        msg: to_binary(&update_lock_msg)?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "update_locks")
        .add_attribute("mint_lock", &locks.mint_lock.to_string())
        .add_attribute("burn_lock", &locks.burn_lock.to_string())
        .add_attribute("transfer_lock", &locks.transfer_lock.to_string())
        .add_attribute("send_lock", &locks.send_lock.to_string()))
}

fn execute_set_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    whitelist: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    config.whitelist = whitelist.and_then(|w| deps.api.addr_validate(w.as_str()).ok());
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "set_whitelist"))
}

fn execute_update_start_time(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    start_time: Option<Timestamp>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if config.start_time.is_some() && env.block.time >= config.start_time.unwrap() {
        return Err(ContractError::AlreadyStarted {});
    }

    match start_time {
        Some(time) => {
            if env.block.time > time {
                return Err(ContractError::InvalidStartTime {});
            }
            config.start_time = start_time;
        }
        None => {
            config.start_time = None;
        }
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_start_time"))
}

fn execute_update_per_address_limit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    per_address_limit: Option<u32>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if per_address_limit.is_some() && per_address_limit.unwrap() == 0 {
        return Err(ContractError::InvalidPerAddressLimit {});
    }

    config.per_address_limit = per_address_limit;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_per_address_limit"))
}

fn execute_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.mint_lock && info.sender != config.admin {
        return Err(ContractError::LockedMint {});
    }

    let token_address = TOKEN_ADDR.load(deps.storage)?;
    let token_id = (TOKEN_ID.load(deps.storage)?) + 1;

    let owner = match recipient {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => info.sender,
    };

    let mint_msg = MintMsg {
        token_id: token_id.to_string(),
        owner: owner.to_string(),
        // TODO: Add token_uri in here
        // We need to pull from ipfs
        token_uri: None,
        // TODO: Maybe even utilize on chain metadata support
        extension: Empty {},
    };
    let msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_address.to_string(),
        msg: to_binary(&TokenExecuteMsg::Mint(mint_msg))?,
        funds: vec![],
    });

    TOKEN_ID.save(deps.storage, &token_id)?;

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "mint"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetTokenAddress {} => to_binary(&query_token_address(deps)?),
    }
}

fn query_token_address(deps: Deps) -> StdResult<TokenAddressResponse> {
    let addr = TOKEN_ADDR.load(deps.storage)?;
    Ok(TokenAddressResponse {
        token_address: addr.to_string(),
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != TOKEN_INSTANTIATE_REPLY_ID {
        return Err(ContractError::InvalidReplyID {});
    }

    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            TOKEN_ADDR.save(deps.storage, &Addr::unchecked(res.contract_address))?;
            Ok(Response::default().add_attribute("action", "instantiate_sg721_reply"))
        }
        Err(_) => Err(ContractError::TokenInstantiateError {}),
    }
}
