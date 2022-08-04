#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use rift_types::marketplace::Listing;
use rift_types::module::Modules;
use rift_types::query::ResponseWrapper;
use rift_utils::{query_collection_address, query_module_address, query_token_owner};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, FixedListing, CONFIG, CONTROLLER_ADDR, FIXED_LISTING};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_MARKETPLACE_FEE_PERCENTAGE: u64 = 5;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = deps.api.addr_validate(&msg.admin)?;

    let config = Config {
        admin,
        fee_percentage: Decimal::percent(DEFAULT_MARKETPLACE_FEE_PERCENTAGE),
    };
    CONFIG.save(deps.storage, &config)?;

    CONTROLLER_ADDR.save(deps.storage, &info.sender);

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
        ExecuteMsg::ListFixedToken {
            collection_id,
            token_id,
            price,
        } => execute_list_fixed_token(deps, env, info, collection_id, token_id, price),
        ExecuteMsg::UpdatePrice {
            listing_type,
            collection_id,
            token_id,
            price,
        } => execute_update_price(
            deps,
            env,
            info,
            listing_type,
            collection_id,
            token_id,
            price,
        ),
    }
}

fn execute_list_fixed_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_id: u32,
    token_id: u32,
    price: Uint128,
) -> Result<Response, ContractError> {
    let owner = get_token_owner(&deps, &collection_id, &token_id)?;

    if owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let fixed_listing = FixedListing {
        collection_id,
        token_id,
        price,
        owner,
    };
    FIXED_LISTING.save(deps.storage, (collection_id, token_id), &fixed_listing)?;

    Ok(Response::new().add_attribute("action", "execute_list_fixed_token"))
}

fn execute_update_price(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    listing_type: Listing,
    collection_id: u32,
    token_id: u32,
    price: Uint128,
) -> Result<Response, ContractError> {
    let owner = get_token_owner(&deps, &collection_id, &token_id)?;

    if owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    match listing_type {
        Listing::Fixed => {
            let mut fixed_listing = FIXED_LISTING.load(deps.storage, (collection_id, token_id))?;
            fixed_listing.price = price;
            FIXED_LISTING.save(deps.storage, (collection_id, token_id), &fixed_listing)?;
        }
        Listing::Auction => unimplemented!(),
    }

    Ok(Response::new().add_attribute("action", "execute_update_price"))
}

fn get_token_owner(
    deps: &DepsMut,
    collection_id: &u32,
    token_id: &u32,
) -> Result<Addr, ContractError> {
    let controller_addr = CONTROLLER_ADDR.load(deps.storage)?;
    let mint_module_addr =
        query_module_address(&deps.querier, &controller_addr, Modules::MintModule)?;
    let collection_addr =
        query_collection_address(&deps.querier, &mint_module_addr, collection_id)?;
    let owner = query_token_owner(&deps.querier, &collection_addr, &token_id)?;
    Ok(owner)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::FixedListing {
            collection_id,
            token_id,
        } => to_binary(&query_fixed_listing(deps, collection_id, token_id)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ResponseWrapper<Config>> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ResponseWrapper::new("config", config))
}

fn query_fixed_listing(
    deps: Deps,
    collection_id: u32,
    token_id: u32,
) -> StdResult<ResponseWrapper<FixedListing>> {
    let listing = FIXED_LISTING.load(deps.storage, (collection_id, token_id))?;
    Ok(ResponseWrapper::new("fixed_listing", listing))
}
