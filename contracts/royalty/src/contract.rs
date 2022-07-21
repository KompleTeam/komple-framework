#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use rift_types::query::ResponseWrapper;
use rift_types::royalty::Royalty;
use rift_utils::query_token_owner;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, COLLECTION_ADDR, CONFIG, OWNER_ROYALTY_ADDR, TOKEN_ROYALTY_ADDR};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:rift-royalty-contract";
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
    if msg.share > Decimal::one() {
        return Err(ContractError::InvalidShare {});
    };

    let config = Config {
        admin: admin.clone(),
        share: msg.share,
        royalty_type: msg.clone().royalty_type,
    };
    CONFIG.save(deps.storage, &config)?;

    COLLECTION_ADDR.save(deps.storage, &info.sender)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateRoyaltyType { royalty_type } => {
            execute_update_royalty_type(deps, env, info, royalty_type)
        }
        ExecuteMsg::UpdateOwnerRoyaltyAddress { address } => {
            execute_update_owner_royalty_address(deps, env, info, address)
        }
        ExecuteMsg::UpdateTokenRoyaltyAddress {
            collection_id,
            token_id,
            address,
        } => {
            execute_update_token_royalty_address(deps, env, info, collection_id, token_id, address)
        }
        ExecuteMsg::UpdateShare { share } => execute_update_share(deps, env, info, share),
    }
}

fn execute_update_royalty_type(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    royalty_type: Royalty,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    config.royalty_type = royalty_type.clone();
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "execute_update_royalty_type")
        .add_attribute("royalty_type", royalty_type.as_str()))
}

fn execute_update_owner_royalty_address(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    // TODO: Need to lock updating the owner royalty address after first time??
    // How will this work think about it

    let addr = deps.api.addr_validate(&address)?;
    OWNER_ROYALTY_ADDR.save(deps.storage, info.sender, &addr)?;

    Ok(Response::new()
        .add_attribute("action", "execute_update_owner_royalty_address")
        .add_attribute("address", address.to_string()))
}

fn execute_update_token_royalty_address(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_id: u32,
    token_id: u32,
    address: String,
) -> Result<Response, ContractError> {
    // TODO: Need to lock updating the owner royalty address after first time??
    // How will this work think about it

    let collection_addr = COLLECTION_ADDR.load(deps.storage)?;
    let owner = query_token_owner(&deps.querier, &collection_addr, &token_id)?;

    if owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let addr = deps.api.addr_validate(&address)?;
    TOKEN_ROYALTY_ADDR.save(deps.storage, (collection_id, token_id), &addr)?;

    Ok(Response::new()
        .add_attribute("action", "execute_update_token_royalty_address")
        .add_attribute("collection_id", collection_id.to_string())
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("address", address.to_string()))
}

fn execute_update_share(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    share: Decimal,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if share > Decimal::one() {
        return Err(ContractError::InvalidShare {});
    };

    config.share = share;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "execute_update_share")
        .add_attribute("share", share.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::RoyaltyAddress {
            owner,
            collection_id,
            token_id,
        } => to_binary(&query_royalty_address(
            deps,
            owner,
            collection_id,
            token_id,
        )?),
    }
}

fn query_config(deps: Deps) -> StdResult<ResponseWrapper<Config>> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ResponseWrapper::new("config", config))
}

fn query_royalty_address(
    deps: Deps,
    owner: String,
    collection_id: u32,
    token_id: u32,
) -> StdResult<ResponseWrapper<String>> {
    let config = CONFIG.load(deps.storage)?;

    let addr = match config.royalty_type {
        Royalty::Admin => config.admin.to_string(),
        Royalty::Owners => {
            let addr =
                OWNER_ROYALTY_ADDR.may_load(deps.storage, deps.api.addr_validate(&owner)?)?;
            match addr {
                Some(addr) => addr.to_string(),
                None => {
                    let collection_addr = COLLECTION_ADDR.load(deps.storage)?;
                    let owner = query_token_owner(&deps.querier, &collection_addr, &token_id)?;
                    owner.to_string()
                }
            }
        }
        Royalty::Tokens => {
            let addr = TOKEN_ROYALTY_ADDR.may_load(deps.storage, (collection_id, token_id))?;
            match addr {
                Some(addr) => addr.to_string(),
                None => {
                    let collection_addr = COLLECTION_ADDR.load(deps.storage)?;
                    let owner = query_token_owner(&deps.querier, &collection_addr, &token_id)?;
                    owner.to_string()
                }
            }
        }
    };
    Ok(ResponseWrapper::new("royalty_address", addr))
}
