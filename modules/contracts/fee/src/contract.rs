use std::ops::Mul;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, BankMsg, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{CustomAddress, ExecuteMsg, InstantiateMsg, QueryMsg, ShareResponse};
use crate::state::{Config, Share, CONFIG, SHARES};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:komple-fee-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        admin: info.sender.clone(),
    };
    CONFIG.save(deps.storage, &config)?;

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
        ExecuteMsg::AddShare {
            name,
            address,
            percentage,
        } => execute_add_share(deps, env, info, name, address, percentage),
        ExecuteMsg::UpdateShare {
            name,
            address,
            percentage,
        } => execute_update_share(deps, env, info, name, address, percentage),
        ExecuteMsg::RemoveShare { name } => execute_remove_share(deps, env, info, name),
        ExecuteMsg::Distribute { custom_addresses } => {
            execute_distribute(deps, env, info, custom_addresses)
        }
    }
}

fn execute_add_share(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    name: String,
    address: Option<String>,
    percentage: Decimal,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if SHARES.has(deps.storage, &name) {
        return Err(ContractError::ExistingShare {});
    }

    if percentage > Decimal::one() {
        return Err(ContractError::InvalidPercentage {});
    }

    let total_fee = query_total_fee(deps.as_ref())?;
    if total_fee + percentage >= Decimal::one() {
        return Err(ContractError::InvalidTotalFee {});
    }

    println!("{:?}", address);

    let share = Share {
        fee_percentage: percentage,
        payment_address: address
            .clone()
            .and_then(|p| deps.api.addr_validate(&p).ok()),
    };

    SHARES.save(deps.storage, &name, &share)?;

    Ok(Response::new()
        .add_attribute("action", "execute_add_share")
        .add_attribute("name", name.to_string())
        // TODO: Figure if we can use none in here
        .add_attribute("address", address.unwrap_or("none".to_string()))
        .add_attribute("percentage", percentage.to_string()))
}

fn execute_update_share(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    name: String,
    address: Option<String>,
    percentage: Decimal,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if !SHARES.has(deps.storage, &name) {
        return Err(ContractError::ShareNotFound {});
    }

    if percentage > Decimal::one() {
        return Err(ContractError::InvalidPercentage {});
    }

    let mut share = SHARES.load(deps.storage, &name)?;

    let total_fee = query_total_fee(deps.as_ref())?;
    if total_fee - share.fee_percentage + percentage >= Decimal::one() {
        return Err(ContractError::InvalidTotalFee {});
    }

    share.fee_percentage = percentage;
    share.payment_address = address
        .clone()
        .and_then(|p| deps.api.addr_validate(&p).ok());
    SHARES.save(deps.storage, &name, &share)?;

    Ok(Response::new()
        .add_attribute("action", "execute_update_share")
        .add_attribute("name", name.to_string())
        .add_attribute("address", address.unwrap_or("".to_string()))
        .add_attribute("percentage", percentage.to_string()))
}

fn execute_remove_share(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    name: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    SHARES.remove(deps.storage, &name);

    Ok(Response::new()
        .add_attribute("action", "execute_remove_share")
        .add_attribute("name", name.to_string()))
}

fn execute_distribute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    custom_addresses: Option<Vec<CustomAddress>>,
) -> Result<Response, ContractError> {
    let mut msgs: Vec<CosmosMsg> = vec![];

    let fund = &info.funds[0];

    // All of the available shares to distribute fee
    let shares = SHARES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (name, share) = item.unwrap();
            ShareResponse {
                name,
                payment_address: share.payment_address.and_then(|p| Some(p.to_string())),
                fee_percentage: share.fee_percentage,
            }
        })
        .collect::<Vec<ShareResponse>>();

    // Total amount of fee percentage
    let total_fee = shares
        .iter()
        .map(|item| item.fee_percentage)
        .sum::<Decimal>()
        .mul(Uint128::new(100));

    // Make bank send messages based on fee percentage
    for share in shares {
        let mut is_custom_address = false;

        // Payment amount is total_funds * percentage / total_fee
        let payment_amount = fund
            .amount
            .mul(share.fee_percentage.mul(Uint128::new(100)))
            .checked_div(total_fee)?;

        // If we have some custom addresses find and replace the payment_address
        if let Some(custom_addresses) = custom_addresses.clone() {
            // Find the correct custom address based on share name
            let custom_share = custom_addresses.iter().find(|item| share.name == item.name);
            if let Some(custom_share) = custom_share {
                is_custom_address = true;
                msgs.push(CosmosMsg::Bank(BankMsg::Send {
                    to_address: custom_share.payment_address.to_string(),
                    amount: vec![coin(payment_amount.u128(), fund.denom.clone())],
                }))
            }
        }

        // Carry on without replacing the addresses
        if !is_custom_address {
            if let Some(payment_address) = share.payment_address {
                msgs.push(CosmosMsg::Bank(BankMsg::Send {
                    to_address: payment_address.to_string(),
                    amount: vec![coin(payment_amount.u128(), fund.denom.clone())],
                }))
            };
        }
    }

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "execute_distribute"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Share { name } => to_binary(&query_share(deps, name)?),
        QueryMsg::Shares {} => to_binary(&query_shares(deps)?),
        QueryMsg::TotalFee {} => to_binary(&query_total_fee(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

fn query_share(deps: Deps, name: String) -> StdResult<ShareResponse> {
    let share = SHARES.load(deps.storage, &name)?;
    Ok(ShareResponse {
        name,
        payment_address: share.payment_address.and_then(|p| Some(p.to_string())),
        fee_percentage: share.fee_percentage,
    })
}

fn query_shares(deps: Deps) -> StdResult<Vec<ShareResponse>> {
    let shares = SHARES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (name, share) = item.unwrap();
            ShareResponse {
                name,
                payment_address: share.payment_address.and_then(|p| Some(p.to_string())),
                fee_percentage: share.fee_percentage,
            }
        })
        .collect::<Vec<ShareResponse>>();
    Ok(shares)
}

fn query_total_fee(deps: Deps) -> StdResult<Decimal> {
    let total_fee = SHARES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (_, share) = item.unwrap();
            share.fee_percentage
        })
        .sum::<Decimal>();
    Ok(total_fee)
}
