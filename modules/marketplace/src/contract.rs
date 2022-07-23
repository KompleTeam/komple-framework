use std::ops::Mul;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Empty, Env,
    MessageInfo, Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use rift_types::marketplace::Listing;
use rift_types::module::Modules;
use rift_types::query::ResponseWrapper;
use rift_utils::{query_collection_address, query_module_address, query_token_owner};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, FixedListing, CONFIG, CONTROLLER_ADDR, FIXED_LISTING};

use token_contract::msg::ExecuteMsg as TokenExecuteMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_MARKETPLACE_FEE_PERCENTAGE: u64 = 5;
const MARKETPLACE_PAYOUT_ADDR: &str = "juno..xxx";

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
        native_denom: msg.native_denom,
    };
    CONFIG.save(deps.storage, &config)?;

    CONTROLLER_ADDR.save(deps.storage, &info.sender)?;

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
        ExecuteMsg::DelistFixedToken {
            collection_id,
            token_id,
        } => execute_delist_fixed_token(deps, env, info, collection_id, token_id),
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
        ExecuteMsg::Buy {
            listing_type,
            collection_id,
            token_id,
        } => execute_buy(deps, env, info, listing_type, collection_id, token_id),
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

    // TODO: Check for token permissions

    let fixed_listing = FixedListing {
        collection_id,
        token_id,
        price,
        owner,
    };
    FIXED_LISTING.save(deps.storage, (collection_id, token_id), &fixed_listing)?;

    Ok(Response::new().add_attribute("action", "execute_list_fixed_token"))
}

fn execute_delist_fixed_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_id: u32,
    token_id: u32,
) -> Result<Response, ContractError> {
    let owner = get_token_owner(&deps, &collection_id, &token_id)?;

    if owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    FIXED_LISTING.remove(deps.storage, (collection_id, token_id));

    Ok(Response::new().add_attribute("action", "execute_delist_fixed_token"))
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

fn execute_buy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    listing_type: Listing,
    collection_id: u32,
    token_id: u32,
) -> Result<Response, ContractError> {
    match listing_type {
        Listing::Fixed => _execute_buy_fixed_listing(&deps, &info, collection_id, token_id),
        Listing::Auction => unimplemented!(),
    }
}

fn _execute_buy_fixed_listing(
    deps: &DepsMut,
    info: &MessageInfo,
    collection_id: u32,
    token_id: u32,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let fixed_listing = FIXED_LISTING.load(deps.storage, (collection_id, token_id))?;

    check_funds(&info, &config, fixed_listing.price)?;

    let controller_addr = CONTROLLER_ADDR.load(deps.storage)?;
    let mint_module_addr =
        query_module_address(&deps.querier, &controller_addr, Modules::MintModule)?;
    let collection_addr =
        query_collection_address(&deps.querier, &mint_module_addr, &collection_id)?;

    let fee = config.fee_percentage.mul(fixed_listing.price);
    let payout = fixed_listing.price.checked_sub(fee)?;

    let owner_payout = BankMsg::Send {
        to_address: fixed_listing.owner.to_string(),
        amount: vec![Coin {
            denom: config.native_denom.to_string(),
            amount: payout,
        }],
    };
    let fee_payout = BankMsg::Send {
        to_address: MARKETPLACE_PAYOUT_ADDR.to_string(),
        amount: vec![Coin {
            denom: config.native_denom.to_string(),
            amount: fee,
        }],
    };
    // TODO: Construct a royalty payout message here if needed
    let transfer_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: collection_addr.to_string(),
        msg: to_binary(&TokenExecuteMsg::TransferNft {
            recipient: info.sender.to_string(),
            token_id: token_id.to_string(),
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_submessages(vec![SubMsg::new(owner_payout), SubMsg::new(fee_payout)])
        .add_message(transfer_msg)
        .add_attribute("action", "execute_buy"))
}

fn check_funds(info: &MessageInfo, config: &Config, price: Uint128) -> Result<(), ContractError> {
    if info.funds.len() != 1 {
        return Err(ContractError::InvalidFunds {});
    };
    let sent_fund = info.funds.get(0).unwrap();
    if sent_fund.denom != config.native_denom {
        return Err(ContractError::InvalidDenom {});
    }
    if sent_fund.amount != price {
        return Err(ContractError::InvalidFunds {});
    }

    Ok(())
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