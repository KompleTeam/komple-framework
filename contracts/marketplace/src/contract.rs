#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Empty, Env,
    MessageInfo, Order, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use cw721_base::msg::{ExecuteMsg as Cw721ExecuteMsg, QueryMsg as Cw721QueryMsg};
use cw_storage_plus::Bound;
use komple_fee_module::msg::{
    CustomPaymentAddress as FeeModuleCustomPaymentAddress, ExecuteMsg as FeeModuleExecuteMsg,
    QueryMsg as FeeModuleQueryMsg,
};
use komple_token_module::{
    msg::{
        ConfigResponse as TokenConfigResponse, ExecuteMsg as TokenExecuteMsg,
        QueryMsg as TokenQueryMsg,
    },
    ContractError as TokenContractError,
};
use komple_types::fee::Fees;
use komple_types::hub::MARBU_FEE_MODULE_NAMESPACE;
use komple_types::marketplace::Listing;
use komple_types::module::Modules;
use komple_types::query::ResponseWrapper;
use komple_types::tokens::Locks;
use komple_utils::{
    funds::check_single_coin, query_collection_address, query_collection_locks,
    query_module_address, query_storage, query_token_locks, query_token_owner,
};
use semver::Version;
use std::ops::Mul;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, FixedListing, CONFIG, FIXED_LISTING, HUB_ADDR};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:komple-marketplace-module";
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

    let config = Config {
        admin,
        native_denom: msg.native_denom,
    };
    CONFIG.save(deps.storage, &config)?;

    HUB_ADDR.save(deps.storage, &info.sender)?;

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

    // Check if the token owner is the same as info.sender
    if owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Checking the collection locks
    let collection_locks = query_collection_locks(&deps.querier, &collection_addr)?;
    check_locks(collection_locks)?;

    // Checking the token locks
    let token_locks = query_token_locks(&deps.querier, &collection_addr, &token_id)?;
    check_locks(token_locks)?;

    let fixed_listing = FixedListing {
        collection_id,
        token_id,
        price,
        owner,
    };
    FIXED_LISTING.save(deps.storage, (collection_id, token_id), &fixed_listing)?;

    // Locking the token so it will not be available for other actions
    let tmp_msg: Cw721ExecuteMsg<Empty, TokenExecuteMsg> = Cw721ExecuteMsg::Extension {
        msg: TokenExecuteMsg::UpdateTokenLock {
            token_id: token_id.to_string(),
            locks: Locks {
                burn_lock: true,
                mint_lock: false,
                transfer_lock: true,
                send_lock: true,
            },
        },
    };
    let lock_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: collection_addr.to_string(),
        msg: to_binary(&tmp_msg).unwrap(),
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

    // Check if the token owner is the same as info.sender
    if owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Throw an error if token is not listed
    // This is needed in case users want to unlock a token
    if !FIXED_LISTING.has(deps.storage, (collection_id, token_id)) {
        return Err(ContractError::NotListed {});
    }
    FIXED_LISTING.remove(deps.storage, (collection_id, token_id));

    // Unlocking token so it can be used again
    let tmp_msg: Cw721ExecuteMsg<Empty, TokenExecuteMsg> = Cw721ExecuteMsg::Extension {
        msg: TokenExecuteMsg::UpdateTokenLock {
            token_id: token_id.to_string(),
            locks: Locks {
                burn_lock: false,
                mint_lock: false,
                transfer_lock: false,
                send_lock: false,
            },
        },
    };
    let unlock_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: collection_addr.to_string(),
        msg: to_binary(&tmp_msg).unwrap(),
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

    // Check if the token owner is the same as info.sender
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
        Listing::Fixed => _execute_buy_fixed_listing(deps, &info, collection_id, token_id),
        Listing::Auction => unimplemented!(),
    }
}

fn _execute_buy_fixed_listing(
    deps: DepsMut,
    info: &MessageInfo,
    collection_id: u32,
    token_id: u32,
) -> Result<Response, ContractError> {
    let hub_addr = HUB_ADDR.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let fixed_listing = FIXED_LISTING.load(deps.storage, (collection_id, token_id))?;

    if fixed_listing.owner == info.sender {
        return Err(ContractError::SelfPurchase {});
    }

    // Check for the sent funds
    check_single_coin(
        &info,
        coin(fixed_listing.price.u128(), config.native_denom.clone()),
    )?;

    let mint_module_addr = query_module_address(&deps.querier, &hub_addr, Modules::Mint)?;
    let collection_addr =
        query_collection_address(&deps.querier, &mint_module_addr, &collection_id)?;

    // Messages to be sent to other contracts
    let mut sub_msgs: Vec<SubMsg> = vec![];

    // Marketplace and royalty fees are 0 at first until they exist
    let mut marketplace_fee = Uint128::zero();
    let mut royalty_fee = Uint128::zero();

    // Process Marbu fee if it exists on Hub
    let res = query_storage::<Addr>(&deps.querier, &hub_addr, MARBU_FEE_MODULE_NAMESPACE)?;
    if let Some(marbu_fee_module) = res {
        // Get the new fee based on fee percentage
        let fee_to_send = get_fee(&deps, &marbu_fee_module, fixed_listing.price);
        marketplace_fee += fee_to_send;
        // Create distribution message and add it to sub_msgss
        let marbu_fee_distribution: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: marbu_fee_module.to_string(),
            msg: to_binary(&FeeModuleExecuteMsg::Distribute {
                fee_type: Fees::Percentage,
                module_name: Modules::Marketplace.to_string(),
                custom_payment_addresses: Some(vec![FeeModuleCustomPaymentAddress {
                    fee_name: "hub_admin".to_string(),
                    address: config.admin.to_string(),
                }]),
            })?,
            funds: vec![Coin {
                denom: config.native_denom.to_string(),
                amount: fee_to_send,
            }],
        });
        sub_msgs.push(SubMsg::new(marbu_fee_distribution));
    };

    // Check if fee module is registerd on the Hub
    let fee_module_addr = query_module_address(&deps.querier, &hub_addr, Modules::Fee);
    if fee_module_addr.is_ok() {
        // Get the new fee based on fee percentage
        let fee_to_send = get_fee(
            &deps,
            &fee_module_addr.as_ref().unwrap(),
            fixed_listing.price,
        );
        marketplace_fee += fee_to_send;
        // Create distribution message and add it to sub_msgss
        let fee_distribution: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: fee_module_addr.unwrap().to_string(),
            msg: to_binary(&FeeModuleExecuteMsg::Distribute {
                fee_type: Fees::Percentage,
                module_name: Modules::Marketplace.to_string(),
                // TODO: Refactor here to accept custom payment addresses
                // This can be updated by admin to take custom fees
                custom_payment_addresses: None,
            })?,
            funds: vec![Coin {
                denom: config.native_denom.to_string(),
                amount: fee_to_send,
            }],
        });
        sub_msgs.push(SubMsg::new(fee_distribution));
    }

    // TODO: Check if this should change with a raw query instead
    // Check if there is a royalty fee add to sub_msgs
    let res: ResponseWrapper<TokenConfigResponse> = deps.querier.query_wasm_smart(
        collection_addr.clone(),
        &Cw721QueryMsg::Extension {
            msg: TokenQueryMsg::Config {},
        },
    )?;
    if let Some(royalty_share) = res.data.royalty_share {
        royalty_fee = royalty_share.mul(fixed_listing.price);

        // Royalty fee message
        let royalty_payout = BankMsg::Send {
            to_address: res.data.creator.to_string(),
            amount: vec![Coin {
                denom: res.data.native_denom.to_string(),
                amount: royalty_fee,
            }],
        };
        sub_msgs.push(SubMsg::new(royalty_payout))
    }

    // Add marketplace and royalty fee and subtract from the price
    let payout = fixed_listing
        .price
        .checked_sub(marketplace_fee + royalty_fee)?;

    // Owner payout message
    let owner_payout = BankMsg::Send {
        to_address: fixed_listing.owner.to_string(),
        amount: vec![Coin {
            denom: config.native_denom.to_string(),
            amount: payout,
        }],
    };
    sub_msgs.push(SubMsg::new(owner_payout));

    // Transfer token ownership to the new address
    let tmp_msg: Cw721ExecuteMsg<Empty, TokenExecuteMsg> = Cw721ExecuteMsg::Extension {
        msg: TokenExecuteMsg::AdminTransferNft {
            recipient: info.sender.to_string(),
            token_id: token_id.to_string(),
        },
    };
    let transfer_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: collection_addr.to_string(),
        msg: to_binary(&tmp_msg)?,
        funds: vec![],
    });

    // Lift up the token locks
    let tmp_msg: Cw721ExecuteMsg<Empty, TokenExecuteMsg> = Cw721ExecuteMsg::Extension {
        msg: TokenExecuteMsg::UpdateTokenLock {
            token_id: token_id.to_string(),
            locks: Locks {
                burn_lock: false,
                mint_lock: false,
                transfer_lock: false,
                send_lock: false,
            },
        },
    };
    let unlock_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: collection_addr.to_string(),
        msg: to_binary(&tmp_msg)?,
        funds: vec![],
    });

    FIXED_LISTING.remove(deps.storage, (collection_id, token_id));

    Ok(Response::new()
        .add_submessages(sub_msgs)
        .add_messages(vec![transfer_msg, unlock_msg])
        .add_attribute("action", "execute_buy"))
}

fn get_fee(deps: &DepsMut, fee_module_addr: &Addr, listing_price: Uint128) -> Uint128 {
    let query = FeeModuleQueryMsg::TotalPercentageFees {
        module_name: Modules::Marketplace.to_string(),
    };
    let res: Result<ResponseWrapper<Decimal>, StdError> =
        deps.querier.query_wasm_smart(fee_module_addr, &query);

    let mut marketplace_fee = Uint128::zero();
    if let Ok(fee_percentage) = res {
        marketplace_fee = fee_percentage.data.mul(listing_price);
    };
    marketplace_fee
}

fn get_collection_address(deps: &DepsMut, collection_id: &u32) -> Result<Addr, ContractError> {
    let hub_addr = HUB_ADDR.load(deps.storage)?;
    let mint_module_addr = query_module_address(&deps.querier, &hub_addr, Modules::Mint)?;
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

/// Gets a single fixed listing
fn query_fixed_listing(
    deps: Deps,
    collection_id: u32,
    token_id: u32,
) -> StdResult<ResponseWrapper<FixedListing>> {
    let listing = FIXED_LISTING.load(deps.storage, (collection_id, token_id))?;
    Ok(ResponseWrapper::new("fixed_listing", listing))
}

/// Gets a batch of fixed listings under a collection
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version: Version = CONTRACT_VERSION.parse()?;
    let contract_version: ContractVersion = get_contract_version(deps.storage)?;
    let storage_version: Version = contract_version.version.parse()?;

    if contract_version.contract != CONTRACT_NAME {
        return Err(
            StdError::generic_err("New version name should match the current version").into(),
        );
    }
    if storage_version >= version {
        return Err(
            StdError::generic_err("New version cannot be smaller than current version").into(),
        );
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}
