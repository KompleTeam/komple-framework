#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Empty, Env,
    MessageInfo, Order, Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use komple_types::marketplace::Listing;
use komple_types::module::Modules;
use komple_types::query::ResponseWrapper;
use komple_types::tokens::Locks;
use komple_utils::{
    check_funds, query_collection_address, query_collection_locks, query_module_address,
    query_storage, query_token_locks, query_token_owner,
};
use std::ops::Mul;
use token_contract::state::Config as TokenConfig;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, FixedListing, CONFIG, CONTROLLER_ADDR, FIXED_LISTING};

use token_contract::{msg::ExecuteMsg as TokenExecuteMsg, ContractError as TokenContractError};

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
    let collection_addr = get_collection_address(&deps, &collection_id)?;
    let owner = query_token_owner(&deps.querier, &collection_addr, &token_id)?;

    if owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let collection_locks = query_collection_locks(&deps.querier, &collection_addr)?;
    check_locks(collection_locks)?;

    let token_locks = query_token_locks(&deps.querier, &collection_addr, &token_id)?;
    check_locks(token_locks)?;

    let fixed_listing = FixedListing {
        collection_id,
        token_id,
        price,
        owner,
    };
    FIXED_LISTING.save(deps.storage, (collection_id, token_id), &fixed_listing)?;

    let lock_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: collection_addr.to_string(),
        msg: to_binary(&TokenExecuteMsg::UpdateTokenLock {
            token_id: token_id.to_string(),
            locks: Locks {
                burn_lock: true,
                mint_lock: false,
                transfer_lock: true,
                send_lock: true,
            },
        })
        .unwrap(),
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(lock_msg)
        .add_attribute("action", "execute_list_fixed_token"))
}

fn execute_delist_fixed_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_id: u32,
    token_id: u32,
) -> Result<Response, ContractError> {
    let collection_addr = get_collection_address(&deps, &collection_id)?;
    let owner = query_token_owner(&deps.querier, &collection_addr, &token_id)?;

    if owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    FIXED_LISTING.remove(deps.storage, (collection_id, token_id));

    let unlock_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: collection_addr.to_string(),
        msg: to_binary(&TokenExecuteMsg::UpdateTokenLock {
            token_id: token_id.to_string(),
            locks: Locks {
                burn_lock: false,
                mint_lock: false,
                transfer_lock: false,
                send_lock: false,
            },
        })
        .unwrap(),
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(unlock_msg)
        .add_attribute("action", "execute_delist_fixed_token"))
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
    let collection_addr = get_collection_address(&deps, &collection_id)?;
    let owner = query_token_owner(&deps.querier, &collection_addr, &token_id)?;

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
    let controller_addr = CONTROLLER_ADDR.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let fixed_listing = FIXED_LISTING.load(deps.storage, (collection_id, token_id))?;

    // Check for the sent funds
    check_funds(&info, &config.native_denom, fixed_listing.price)?;

    let mint_module_addr =
        query_module_address(&deps.querier, &controller_addr, Modules::MintModule)?;
    let collection_addr =
        query_collection_address(&deps.querier, &mint_module_addr, &collection_id)?;

    // This is the fee for marketplace
    let fee = config.fee_percentage.mul(fixed_listing.price);

    // This is the fee for royalty owner
    // Zero at first because it royalty might not exist
    let mut royalty_fee = Uint128::new(0);

    let mut sub_msgs: Vec<SubMsg> = vec![];

    // Get royalty message if it exists
    let res = query_storage::<TokenConfig>(&deps.querier, &collection_addr, "config")?;
    if let Some(config) = res {
        if config.royalty_share.is_some() {
            royalty_fee = config.royalty_share.unwrap().mul(fixed_listing.price);

            let royalty_payout = BankMsg::Send {
                to_address: config.admin.to_string(),
                amount: vec![Coin {
                    denom: config.native_denom.to_string(),
                    amount: royalty_fee,
                }],
            };
            sub_msgs.push(SubMsg::new(royalty_payout))
        }
    }

    // Add marketplace and royalty fee and subtract from the price
    let payout = fixed_listing.price.checked_sub(fee + royalty_fee)?;

    let fee_payout = BankMsg::Send {
        to_address: MARKETPLACE_PAYOUT_ADDR.to_string(),
        amount: vec![Coin {
            denom: config.native_denom.to_string(),
            amount: fee,
        }],
    };
    let owner_payout = BankMsg::Send {
        to_address: fixed_listing.owner.to_string(),
        amount: vec![Coin {
            denom: config.native_denom.to_string(),
            amount: payout,
        }],
    };
    sub_msgs.push(SubMsg::new(owner_payout));
    sub_msgs.push(SubMsg::new(fee_payout));

    // Transfer token ownership to the new address
    let transfer_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: collection_addr.to_string(),
        msg: to_binary(&TokenExecuteMsg::AdminTransferNft {
            recipient: info.sender.to_string(),
            token_id: token_id.to_string(),
        })?,
        funds: vec![],
    });

    // Lift up the token locks
    let unlock_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: collection_addr.to_string(),
        msg: to_binary(&TokenExecuteMsg::UpdateTokenLock {
            token_id: token_id.to_string(),
            locks: Locks {
                burn_lock: false,
                mint_lock: false,
                transfer_lock: false,
                send_lock: false,
            },
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_submessages(sub_msgs)
        .add_messages(vec![transfer_msg, unlock_msg])
        .add_attribute("action", "execute_buy"))
}

fn get_collection_address(deps: &DepsMut, collection_id: &u32) -> Result<Addr, ContractError> {
    let controller_addr = CONTROLLER_ADDR.load(deps.storage)?;
    let mint_module_addr =
        query_module_address(&deps.querier, &controller_addr, Modules::MintModule)?;
    let collection_addr =
        query_collection_address(&deps.querier, &mint_module_addr, collection_id)?;
    Ok(collection_addr)
}

fn check_locks(locks: Locks) -> Result<(), TokenContractError> {
    if locks.transfer_lock {
        return Err(TokenContractError::TransferLocked {}.into());
    };
    if locks.send_lock {
        return Err(TokenContractError::SendLocked {}.into());
    };
    if locks.burn_lock {
        return Err(TokenContractError::BurnLocked {}.into());
    };
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::FixedListing {
            collection_id,
            token_id,
        } => to_binary(&query_fixed_listing(deps, collection_id, token_id)?),
        QueryMsg::FixedListings {
            collection_id,
            start_after,
            limit,
        } => to_binary(&query_fixed_listings(
            deps,
            collection_id,
            start_after,
            limit,
        )?),
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

fn query_fixed_listings(
    deps: Deps,
    collection_id: u32,
    start_after: Option<u32>,
    limit: Option<u32>,
) -> StdResult<ResponseWrapper<Vec<FixedListing>>> {
    let limit = limit.unwrap_or(30) as usize;
    let start = start_after.map(Bound::exclusive);
    let listings = FIXED_LISTING
        .prefix(collection_id)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_, listing) = item.unwrap();
            listing
        })
        .collect::<Vec<FixedListing>>();

    Ok(ResponseWrapper::new("listings", listings))
}
