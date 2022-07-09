#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult, Timestamp,
};
use cw2::set_contract_version;
use url::Url;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, LocksReponse, MintedTokenAmountResponse, QueryMsg};
use crate::state::{
    CollectionInfo, Config, Locks, COLLECTION_INFO, CONFIG, LOCKS, MINTED_TOKEN_AMOUNTS, TOKEN_IDS,
    TOKEN_LOCKS,
};

use cw721::{ContractInfoResponse, Cw721Execute};
use cw721_base::MintMsg;

pub type Cw721Contract<'a> = cw721_base::Cw721Contract<'a, Empty, Empty>;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_DESCRIPTION_LENGTH: u32 = 512;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let locks = Locks {
        burn_lock: false,
        mint_lock: false,
        transfer_lock: false,
        send_lock: false,
    };

    let whitelist = msg
        .whitelist
        .and_then(|w| deps.api.addr_validate(w.as_str()).ok());

    if msg.start_time.is_some() && env.block.time > msg.start_time.unwrap() {
        return Err(ContractError::InvalidStartTime {});
    }

    let config = Config {
        admin: info.sender,
        per_address_limit: msg.per_address_limit,
        start_time: msg.start_time,
        whitelist,
    };
    CONFIG.save(deps.storage, &config)?;

    LOCKS.save(deps.storage, &locks)?;

    TOKEN_IDS.save(deps.storage, &0)?;

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

    let contract_info = ContractInfoResponse {
        name: msg.token_info.name.clone(),
        symbol: msg.token_info.symbol,
    };
    Cw721Contract::default()
        .contract_info
        .save(deps.storage, &contract_info)?;

    let minter = deps.api.addr_validate(&msg.token_info.minter)?;
    Cw721Contract::default()
        .minter
        .save(deps.storage, &minter)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("minter", msg.token_info.minter)
        .add_attribute("collection_name", msg.token_info.name))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<Empty>,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateLocks { locks } => execute_update_locks(deps, env, info, locks),
        ExecuteMsg::UpdateTokenLock { token_id, locks } => {
            execute_update_token_locks(deps, env, info, token_id, locks)
        }
        ExecuteMsg::Mint(mint_msg) => execute_mint(deps, env, info, mint_msg),
        ExecuteMsg::Burn { token_id } => execute_burn(deps, env, info, token_id),
        ExecuteMsg::TransferNft {
            token_id,
            recipient,
        } => execute_transfer(deps, env, info, token_id, recipient),
        ExecuteMsg::SendNft {
            token_id,
            contract,
            msg,
        } => execute_send(deps, env, info, token_id, contract, msg),
        ExecuteMsg::UpdatePerAddressLimit { per_address_limit } => {
            execute_update_per_address_limit(deps, env, info, per_address_limit)
        }
        ExecuteMsg::UpdateStartTime { start_time } => {
            execute_update_start_time(deps, env, info, start_time)
        }
        ExecuteMsg::SetWhitelist { whitelist } => execute_set_whitelist(deps, env, info, whitelist),
        _ => {
            let res = Cw721Contract::default().execute(deps, env, info, msg.into());
            match res {
                Ok(res) => Ok(res),
                Err(e) => Err(e.into()),
            }
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

    LOCKS.save(deps.storage, &locks)?;

    Ok(Response::new()
        .add_attribute("action", "update_locks")
        .add_attribute("mint_lock", locks.mint_lock.to_string())
        .add_attribute("burn_lock", locks.burn_lock.to_string())
        .add_attribute("transfer_lock", locks.transfer_lock.to_string())
        .add_attribute("send_lock", locks.send_lock.to_string()))
}

pub fn execute_update_token_locks(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
    locks: Locks,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    TOKEN_LOCKS.save(deps.storage, &token_id, &locks)?;

    Ok(Response::new()
        .add_attribute("action", "update_token_locks")
        .add_attribute("token_id", token_id)
        .add_attribute("mint_lock", locks.mint_lock.to_string())
        .add_attribute("burn_lock", locks.burn_lock.to_string())
        .add_attribute("transfer_lock", locks.transfer_lock.to_string())
        .add_attribute("send_lock", locks.send_lock.to_string()))
}

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mint_msg: MintMsg<Empty>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let locks = LOCKS.load(deps.storage)?;
    if locks.mint_lock {
        return Err(ContractError::MintLocked {});
    }

    let token_lock = TOKEN_LOCKS.may_load(deps.storage, &mint_msg.token_id)?;
    if token_lock.is_some() && token_lock.unwrap().mint_lock {
        return Err(ContractError::MintLocked {});
    }

    let total_minted = MINTED_TOKEN_AMOUNTS
        .may_load(deps.storage, &mint_msg.owner)?
        .unwrap_or(0);

    if config.per_address_limit.is_some() && total_minted + 1 > config.per_address_limit.unwrap() {
        return Err(ContractError::TokenLimitReached {});
    }

    // TODO: Check for start time here

    MINTED_TOKEN_AMOUNTS.save(deps.storage, &mint_msg.owner, &(total_minted + 1))?;

    let res = Cw721Contract::default().mint(deps, env, info, mint_msg);
    match res {
        Ok(res) => Ok(res),
        Err(e) => Err(e.into()),
    }
}

pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let locks = LOCKS.load(deps.storage)?;
    if locks.burn_lock {
        return Err(ContractError::BurnLocked {});
    }

    let token_lock = TOKEN_LOCKS.may_load(deps.storage, &token_id)?;
    if token_lock.is_some() && token_lock.unwrap().burn_lock {
        return Err(ContractError::BurnLocked {});
    }

    let res = Cw721Contract::default().burn(deps, env, info, token_id);
    match res {
        Ok(res) => Ok(res),
        Err(e) => Err(e.into()),
    }
}

pub fn execute_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    recipient: String,
) -> Result<Response, ContractError> {
    let locks = LOCKS.load(deps.storage)?;
    if locks.transfer_lock {
        return Err(ContractError::TransferLocked {});
    }

    let token_lock = TOKEN_LOCKS.may_load(deps.storage, &token_id)?;
    if token_lock.is_some() && token_lock.unwrap().transfer_lock {
        return Err(ContractError::TransferLocked {});
    }

    let res = Cw721Contract::default().transfer_nft(deps, env, info, recipient, token_id);
    match res {
        Ok(res) => Ok(res),
        Err(e) => Err(e.into()),
    }
}

pub fn execute_send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    contract: String,
    msg: Binary,
) -> Result<Response, ContractError> {
    let locks = LOCKS.load(deps.storage)?;
    if locks.send_lock {
        return Err(ContractError::SendLocked {});
    }

    let token_lock = TOKEN_LOCKS.may_load(deps.storage, &token_id)?;
    if token_lock.is_some() && token_lock.unwrap().send_lock {
        return Err(ContractError::SendLocked {});
    }

    let res = Cw721Contract::default().send_nft(deps, env, info, contract, token_id, msg);
    match res {
        Ok(res) => Ok(res),
        Err(e) => Err(e.into()),
    }
}

pub fn execute_update_per_address_limit(
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
        None => config.start_time = None,
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_start_time"))
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Locks {} => to_binary(&query_locks(deps)?),
        QueryMsg::TokenLocks { token_id } => to_binary(&query_token_locks(deps, token_id)?),
        QueryMsg::MintedTokenAmount { address } => {
            to_binary(&query_minted_token_amount(deps, address)?)
        }
        QueryMsg::CollectionInfo {} => to_binary(&query_collection_info(deps)?),
        _ => Cw721Contract::default().query(deps, env, msg.into()),
    }
}

fn query_locks(deps: Deps) -> StdResult<LocksReponse> {
    let locks = LOCKS.load(deps.storage)?;
    Ok(LocksReponse { locks })
}

fn query_token_locks(deps: Deps, token_id: String) -> StdResult<LocksReponse> {
    let locks = TOKEN_LOCKS.load(deps.storage, &token_id)?;
    Ok(LocksReponse { locks })
}

fn query_minted_token_amount(deps: Deps, address: String) -> StdResult<MintedTokenAmountResponse> {
    let amount = MINTED_TOKEN_AMOUNTS
        .may_load(deps.storage, &address)?
        .unwrap_or(0);
    Ok(MintedTokenAmountResponse { amount })
}

fn query_collection_info(deps: Deps) -> StdResult<CollectionInfo> {
    let collection_info = COLLECTION_INFO.load(deps.storage)?;
    Ok(collection_info)
}
