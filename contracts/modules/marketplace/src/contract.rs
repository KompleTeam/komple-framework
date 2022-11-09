#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, from_binary, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Order, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw_storage_plus::Bound;
use komple_fee_module::{
    helper::KompleFeeModule, msg::CustomPaymentAddress as FeeModuleCustomPaymentAddress,
    msg::ExecuteMsg as FeeModuleExecuteMsg,
};
use komple_token_module::{
    helper::KompleTokenModule, state::Config as TokenConfig, ContractError as TokenContractError,
};
use komple_types::fee::FundInfo;
use komple_types::marketplace::Listing;
use komple_types::module::Modules;
use komple_types::query::ResponseWrapper;
use komple_types::shared::RegisterMsg;
use komple_types::token::Locks;
use komple_types::{fee::Fees, shared::CONFIG_NAMESPACE};
use komple_types::{
    fee::{MarketplaceFees, MintFees},
    hub::MARBU_FEE_MODULE_NAMESPACE,
};
use komple_utils::funds::{check_cw20_fund_info, FundsError};
use komple_utils::response::ResponseHelper;
use komple_utils::shared::{execute_lock_execute, execute_update_operators};
use komple_utils::{
    check_admin_privileges, funds::check_single_coin, response::EventHelper, storage::StorageHelper,
};
use semver::Version;
use std::ops::Mul;

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, ReceiveMsg};
use crate::state::{
    Config, FixedListing, CONFIG, EXECUTE_LOCK, FIXED_LISTING, FUND_INFO, HUB_ADDR,
};
use crate::{error::ContractError, state::OPERATORS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:komple-marketplace-module";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: RegisterMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Return error if instantiate data is not sent
    if msg.data.is_none() {
        return Err(ContractError::InvalidInstantiateMsg {});
    };
    let data: InstantiateMsg = from_binary(&msg.data.unwrap())?;

    let admin = deps.api.addr_validate(&msg.admin)?;
    let config = Config {
        admin,
        buy_lock: false,
    };
    CONFIG.save(deps.storage, &config)?;

    HUB_ADDR.save(deps.storage, &info.sender)?;

    EXECUTE_LOCK.save(deps.storage, &false)?;

    let cw20_address = match data.fund_info.cw20_address {
        Some(addr) => Some(deps.api.addr_validate(&addr)?),
        None => None,
    };
    let fund_info = FundInfo {
        is_native: data.fund_info.is_native,
        denom: data.fund_info.denom,
        cw20_address: cw20_address.clone(),
    };

    if !fund_info.is_native {
        if cw20_address.is_none() {
            return Err(FundsError::InvalidCw20Token {}.into());
        };
        check_cw20_fund_info(&deps, &fund_info)?;
    };
    FUND_INFO.save(deps.storage, &fund_info)?;

    Ok(
        ResponseHelper::new_module("marketplace", "instantiate").add_event(
            EventHelper::new("marketplace_instantiate")
                .add_attribute("admin", config.admin)
                .add_attribute("is_native", fund_info.is_native.to_string())
                .add_attribute("denom", fund_info.denom.to_string())
                .check_add_attribute(
                    &fund_info.cw20_address,
                    "cw20_address",
                    fund_info
                        .cw20_address
                        .as_ref()
                        .unwrap_or(&Addr::unchecked("")),
                )
                .add_attribute("hub_addr", info.sender)
                .get(),
        ),
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let execute_lock = EXECUTE_LOCK.load(deps.storage)?;
    if execute_lock {
        return Err(ContractError::ExecuteLocked {});
    };

    match msg {
        ExecuteMsg::UpdateBuyLock { lock } => update_buy_lock(deps, env, info, lock),
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
        ExecuteMsg::PermissionBuy {
            listing_type,
            collection_id,
            token_id,
            buyer,
        } => execute_permission_buy(
            deps,
            env,
            info,
            listing_type,
            collection_id,
            token_id,
            buyer,
        ),
        ExecuteMsg::UpdateOperators { addrs } => {
            let config = CONFIG.load(deps.storage)?;
            let res = execute_update_operators(
                deps,
                info,
                "marketplace",
                &env.contract.address,
                &config.admin,
                OPERATORS,
                addrs,
            );
            match res {
                Ok(res) => Ok(res),
                Err(err) => Err(err.into()),
            }
        }
        ExecuteMsg::LockExecute {} => {
            let res = execute_lock_execute(
                deps,
                info,
                "marketplace",
                &env.contract.address,
                EXECUTE_LOCK,
            );
            match res {
                Ok(res) => Ok(res),
                Err(err) => Err(err.into()),
            }
        }
        ExecuteMsg::Receive(msg) => execute_receive(deps, env, info, msg),
    }
}

fn update_buy_lock(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    lock: bool,
) -> Result<Response, ContractError> {
    let hub_addr = HUB_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let mut config = CONFIG.load(deps.storage)?;
    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        hub_addr,
        operators,
    )?;

    config.buy_lock = lock;
    CONFIG.save(deps.storage, &config)?;

    Ok(
        ResponseHelper::new_module("marketplace", "update_buy_lock").add_event(
            EventHelper::new("marketplace_buy_lock")
                .add_attribute("lock", lock.to_string())
                .get(),
        ),
    )
}

fn execute_list_fixed_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_id: u32,
    token_id: u32,
    price: Uint128,
) -> Result<Response, ContractError> {
    let fixed_listing = FIXED_LISTING.may_load(deps.storage, (collection_id, token_id))?;
    if fixed_listing.is_some() {
        return Err(ContractError::AlreadyListed {});
    };

    let collection_addr = get_collection_address(&deps, &collection_id)?;
    let owner = StorageHelper::query_token_owner(&deps.querier, &collection_addr, &token_id)?;

    // Check if the token owner is the same as info.sender
    if owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Checking the collection locks
    let collection_locks = StorageHelper::query_collection_locks(&deps.querier, &collection_addr)?;
    check_locks(collection_locks)?;

    // Checking the token locks
    let token_locks = StorageHelper::query_token_locks(&deps.querier, &collection_addr, &token_id)?;
    check_locks(token_locks)?;

    // Create the fixed listing
    let fixed_listing = FixedListing {
        collection_id,
        token_id,
        price,
        owner,
    };
    FIXED_LISTING.save(deps.storage, (collection_id, token_id), &fixed_listing)?;

    // Locking the token so it will not be available for other actions
    let lock_msg = KompleTokenModule(collection_addr).update_token_locks_msg(
        token_id.to_string(),
        Locks {
            burn_lock: true,
            mint_lock: false,
            transfer_lock: true,
            send_lock: true,
        },
    )?;

    Ok(
        ResponseHelper::new_module("marketplace", "list_fixed_token")
            .add_message(lock_msg)
            .add_event(
                EventHelper::new("marketplace_list_fixed_token")
                    .add_attribute("collection_id", collection_id.to_string())
                    .add_attribute("token_id", token_id.to_string())
                    .add_attribute("price", price.to_string())
                    .get(),
            ),
    )
}

fn execute_delist_fixed_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_id: u32,
    token_id: u32,
) -> Result<Response, ContractError> {
    let collection_addr = get_collection_address(&deps, &collection_id)?;
    let owner = StorageHelper::query_token_owner(&deps.querier, &collection_addr, &token_id)?;

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
    let unlock_msg = KompleTokenModule(collection_addr).update_token_locks_msg(
        token_id.to_string(),
        Locks {
            burn_lock: false,
            mint_lock: false,
            transfer_lock: false,
            send_lock: false,
        },
    )?;

    Ok(
        ResponseHelper::new_module("marketplace", "delist_fixed_token")
            .add_message(unlock_msg)
            .add_event(
                EventHelper::new("marketplace_delist_fixed_token")
                    .add_attribute("collection_id", collection_id.to_string())
                    .add_attribute("token_id", token_id.to_string())
                    .get(),
            ),
    )
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
    let owner = StorageHelper::query_token_owner(&deps.querier, &collection_addr, &token_id)?;

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

    Ok(
        ResponseHelper::new_module("marketplace", "update_price").add_event(
            EventHelper::new("marketplace_update_price")
                .add_attribute("listing_type", listing_type.to_string())
                .add_attribute("collection_id", collection_id.to_string())
                .add_attribute("token_id", token_id.to_string())
                .add_attribute("price", price.to_string())
                .get(),
        ),
    )
}

fn execute_buy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    listing_type: Listing,
    collection_id: u32,
    token_id: u32,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.buy_lock {
        return Err(ContractError::BuyLocked {});
    };

    match listing_type {
        Listing::Fixed => _execute_buy_fixed_listing(
            deps,
            &info,
            collection_id,
            token_id,
            info.sender.to_string(),
            None,
        ),
        Listing::Auction => unimplemented!(),
    }
}

fn execute_permission_buy(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    listing_type: Listing,
    collection_id: u32,
    token_id: u32,
    buyer: String,
) -> Result<Response, ContractError> {
    let hub_addr = HUB_ADDR.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        hub_addr,
        operators,
    )?;

    match listing_type {
        Listing::Fixed => {
            _execute_buy_fixed_listing(deps, &info, collection_id, token_id, buyer, None)
        }
        Listing::Auction => unimplemented!(),
    }
}

fn _execute_buy_fixed_listing(
    deps: DepsMut,
    info: &MessageInfo,
    collection_id: u32,
    token_id: u32,
    buyer: String,
    cw20_token_amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let hub_addr = HUB_ADDR.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let fixed_listing = FIXED_LISTING.load(deps.storage, (collection_id, token_id))?;
    let fund_info = FUND_INFO.load(deps.storage)?;

    // If owner and the buyer is the same return error
    if fixed_listing.owner == buyer {
        return Err(ContractError::SelfPurchase {});
    }

    match fund_info.is_native {
        true => check_single_coin(
            info,
            coin(fixed_listing.price.u128(), fund_info.denom.clone()),
        )?,
        false => {
            if cw20_token_amount.is_none() && cw20_token_amount.unwrap() != fixed_listing.price {
                return Err(FundsError::InvalidCw20Token {}.into());
            };
        }
    }

    // Get the collection address
    let collection_addr = get_collection_address(&deps, &collection_id)?;

    // Messages to be sent to other contracts
    let mut sub_msgs: Vec<SubMsg> = vec![];

    // Marketplace and royalty fees are 0 at first until they exist
    let mut marketplace_fee = Uint128::zero();
    let mut royalty_fee = Uint128::zero();

    // Process Marbu fee if exists on Hub
    let res =
        StorageHelper::query_storage::<Addr>(&deps.querier, &hub_addr, MARBU_FEE_MODULE_NAMESPACE)?;
    if let Some(marbu_fee_module) = res {
        process_marketplace_fees(
            &deps,
            &fund_info,
            &mut sub_msgs,
            &marbu_fee_module,
            fixed_listing.price,
            &mut marketplace_fee,
            Some(vec![FeeModuleCustomPaymentAddress {
                fee_name: MarketplaceFees::HubAdmin.as_str().to_string(),
                address: config.admin.to_string(),
            }]),
        )?;
    };

    // Process fee module fees if exists on Hub
    let fee_module_addr =
        StorageHelper::query_module_address(&deps.querier, &hub_addr, Modules::Fee.to_string());
    if let Ok(fee_module_addr) = fee_module_addr {
        // Marketplace fees
        // process_marketplace_fees(
        //     &deps,
        //     &config,
        //     &mut sub_msgs,
        //     &fee_module_addr,
        //     fixed_listing.price,
        //     &mut marketplace_fee,
        //     None,
        // )?;

        // Collection royalty fees
        let res = StorageHelper::query_percentage_fee(
            &deps.querier,
            &fee_module_addr,
            Modules::Mint.to_string(),
            MintFees::new_royalty(collection_id),
        );
        if let Ok(percentage_fee) = res {
            royalty_fee = percentage_fee.value.mul(fixed_listing.price);

            let res = StorageHelper::query_storage::<TokenConfig>(
                &deps.querier,
                &collection_addr,
                CONFIG_NAMESPACE,
            )?;
            if let Some(token_config) = res {
                let royalty_payout = match fund_info.is_native {
                    true => CosmosMsg::Bank(BankMsg::Send {
                        to_address: token_config.creator.to_string(),
                        amount: vec![Coin {
                            denom: fund_info.denom.to_string(),
                            amount: royalty_fee,
                        }],
                    }),
                    false => CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: fund_info.cw20_address.as_ref().unwrap().to_string(),
                        msg: to_binary(&Cw20ExecuteMsg::Transfer {
                            recipient: token_config.creator.to_string(),
                            amount: royalty_fee,
                        })?,
                        funds: vec![],
                    }),
                };
                sub_msgs.push(SubMsg::new(royalty_payout))
            };
        };
    };

    // Add marketplace and royalty fee and subtract from the price
    let payout = fixed_listing
        .price
        .checked_sub(marketplace_fee + royalty_fee)?;

    // Owner payout message
    let owner_payout = match fund_info.is_native {
        true => CosmosMsg::Bank(BankMsg::Send {
            to_address: fixed_listing.owner.to_string(),
            amount: vec![Coin {
                denom: fund_info.denom.to_string(),
                amount: payout,
            }],
        }),
        false => CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: fund_info.cw20_address.unwrap().to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: fixed_listing.owner.to_string(),
                amount: payout,
            })?,
            funds: vec![],
        }),
    };
    sub_msgs.push(SubMsg::new(owner_payout));

    // Transfer token ownership to the new address
    let transfer_msg = KompleTokenModule(collection_addr.clone())
        .admin_transfer_nft_msg(token_id.to_string(), buyer.clone())?;

    // Lift up the token locks
    let unlock_msg = KompleTokenModule(collection_addr).update_token_locks_msg(
        token_id.to_string(),
        Locks {
            burn_lock: false,
            mint_lock: false,
            transfer_lock: false,
            send_lock: false,
        },
    )?;

    FIXED_LISTING.remove(deps.storage, (collection_id, token_id));

    Ok(ResponseHelper::new_module("marketplace", "buy")
        .add_submessages(sub_msgs)
        .add_messages(vec![transfer_msg, unlock_msg])
        .add_event(
            EventHelper::new("marketplace_buy")
                .add_attribute("listing_type", "fixed")
                .add_attribute("collection_id", collection_id.to_string())
                .add_attribute("token_id", token_id.to_string())
                .add_attribute("price", fixed_listing.price.to_string())
                .add_attribute("owner", fixed_listing.owner)
                .add_attribute("buyer", buyer)
                .add_attribute("marketplace_fee", marketplace_fee.to_string())
                .add_attribute("royalty_fee", royalty_fee.to_string())
                .add_attribute("payout", payout.to_string())
                .get(),
        ))
}

fn execute_receive(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cw20_receive_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: ReceiveMsg = from_binary(&cw20_receive_msg.msg)?;
    let sender = cw20_receive_msg.sender;
    let amount = cw20_receive_msg.amount;
    match msg {
        ReceiveMsg::Buy {
            listing_type,
            collection_id,
            token_id,
        } => {
            let config = CONFIG.load(deps.storage)?;
            if config.buy_lock {
                return Err(ContractError::BuyLocked {});
            };

            match listing_type {
                Listing::Fixed => _execute_buy_fixed_listing(
                    deps,
                    &info,
                    collection_id,
                    token_id,
                    sender,
                    Some(amount),
                ),
                Listing::Auction => unimplemented!(),
            }
        }
    }
}

// Gets the current total fee percentage from fee module
// If exists updates the marketplace fee
// Creates a distribute msg and adds to sub message
fn process_marketplace_fees(
    deps: &DepsMut,
    fund_info: &FundInfo,
    sub_msgs: &mut Vec<SubMsg>,
    fee_module_addr: &Addr,
    listing_price: Uint128,
    marketplace_fee: &mut Uint128,
    custom_payment_addresses: Option<Vec<FeeModuleCustomPaymentAddress>>,
) -> Result<(), ContractError> {
    let fee_percentage = KompleFeeModule(fee_module_addr.to_owned())
        .query_total_percentage_fees(&deps.querier, Modules::Marketplace.as_str())?;

    if !fee_percentage.is_zero() {
        let fee_to_send = fee_percentage.mul(listing_price);

        if !fee_to_send.is_zero() {
            *marketplace_fee += fee_to_send;

            // Create distribution message and add it to sub_msgs
            if fund_info.is_native {
                sub_msgs.push(SubMsg::new(
                    KompleFeeModule(fee_module_addr.to_owned()).distribute_msg(
                        Fees::Percentage,
                        Modules::Marketplace.to_string(),
                        custom_payment_addresses,
                        vec![Coin {
                            denom: fund_info.denom.to_string(),
                            amount: fee_to_send,
                        }],
                    )?,
                ));
            } else {
                sub_msgs.push(SubMsg::new(WasmMsg::Execute {
                    contract_addr: fund_info.cw20_address.as_ref().unwrap().to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        contract: fee_module_addr.to_string(),
                        amount: fee_to_send,
                        msg: to_binary(&FeeModuleExecuteMsg::Distribute {
                            fee_type: Fees::Percentage,
                            module_name: Modules::Marketplace.to_string(),
                            custom_payment_addresses,
                        })?,
                    })?,
                    funds: vec![],
                }));
            }
        }
    };

    Ok(())
}

fn get_collection_address(deps: &DepsMut, collection_id: &u32) -> Result<Addr, ContractError> {
    let hub_addr = HUB_ADDR.load(deps.storage)?;
    let mint_module_addr =
        StorageHelper::query_module_address(&deps.querier, &hub_addr, Modules::Mint.to_string())?;
    let collection_addr =
        StorageHelper::query_collection_address(&deps.querier, &mint_module_addr, collection_id)?;
    Ok(collection_addr)
}

fn check_locks(locks: Locks) -> Result<(), TokenContractError> {
    if locks.transfer_lock {
        return Err(TokenContractError::TransferLocked {});
    };
    if locks.send_lock {
        return Err(TokenContractError::SendLocked {});
    };
    if locks.burn_lock {
        return Err(TokenContractError::BurnLocked {});
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
        QueryMsg::Operators {} => to_binary(&query_operators(deps)?),
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

fn query_operators(deps: Deps) -> StdResult<ResponseWrapper<Vec<String>>> {
    let addrs = OPERATORS.may_load(deps.storage)?;
    let addrs = match addrs {
        Some(addrs) => addrs.iter().map(|a| a.to_string()).collect(),
        None => vec![],
    };
    Ok(ResponseWrapper::new("operators", addrs))
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
