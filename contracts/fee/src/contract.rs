use std::ops::Mul;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, from_binary, to_binary, BankMsg, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, Event,
    MessageInfo, Order, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use komple_types::fee::Fees;
use komple_types::module::Modules;
use komple_types::query::ResponseWrapper;
use komple_utils::check_admin_privileges;
use komple_utils::funds::{check_single_amount, FundsError};

use crate::error::ContractError;
use crate::msg::{
    CustomPaymentAddress, ExecuteMsg, FixedFeeResponse, InstantiateMsg, PercentageFeeResponse,
    QueryMsg,
};
use crate::state::{
    Config, FixedPayment, PercentagePayment, CONFIG, FIXED_FEES, HUB_ADDR, PERCENTAGE_FEES,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:komple-fee-module";
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

    let config = Config { admin };
    CONFIG.save(deps.storage, &config)?;

    HUB_ADDR.save(deps.storage, &info.sender)?;

    Ok(Response::new().add_event(
        Event::new("komple_framework")
            .add_attribute("module", Modules::Fee.as_str())
            .add_attribute("action", "instantiate")
            .add_attribute("admin", info.sender),
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetFee {
            fee_type,
            module_name,
            fee_name,
            data,
        } => execute_set_fee(deps, env, info, fee_type, module_name, fee_name, data),
        ExecuteMsg::RemoveFee {
            fee_type,
            module_name,
            fee_name,
        } => execute_remove_fee(deps, env, info, fee_type, module_name, fee_name),
        ExecuteMsg::Distribute {
            fee_type,
            module_name,
            custom_payment_addresses,
        } => execute_distribute(
            deps,
            env,
            info,
            fee_type,
            module_name,
            custom_payment_addresses,
        ),
    }
}

fn execute_set_fee(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    fee_type: Fees,
    module_name: String,
    fee_name: String,
    data: Binary,
) -> Result<Response, ContractError> {
    let hub_addr = HUB_ADDR.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        hub_addr,
        None,
    )?;

    let mut event_attributes: Vec<(&str, String)> = vec![];

    match fee_type {
        Fees::Fixed => {
            let fixed_payment: FixedPayment = from_binary(&data)?;
            if fixed_payment.value.is_zero() {
                return Err(ContractError::InvalidFee {});
            };

            if fixed_payment.address.is_some() {
                deps.api
                    .addr_validate(&fixed_payment.address.clone().unwrap())?;
            }

            FIXED_FEES.save(deps.storage, (&module_name, &fee_name), &fixed_payment)?;

            event_attributes.push(("value", fixed_payment.value.to_string()));
            if let Some(payment_address) = fixed_payment.address {
                event_attributes.push(("address", payment_address.to_string()));
            }
        }
        Fees::Percentage => {
            let percentage_payment: PercentagePayment = from_binary(&data)?;
            if percentage_payment.value > Decimal::one() {
                return Err(ContractError::InvalidFee {});
            };

            // Query total fee percentage for a given module
            // Total fee decimal cannot be equal or higher than 1
            // If we have an existing state, we need to subtract the current value before additions
            let current_percentage_payment =
                PERCENTAGE_FEES.may_load(deps.storage, (&module_name, &fee_name))?;
            let current_percentage_payment_value = match current_percentage_payment {
                Some(p) => p.value,
                None => Decimal::zero(),
            };
            let total_fee = query_total_percentage_fees(deps.as_ref(), module_name.clone())?;
            if total_fee.data - current_percentage_payment_value + percentage_payment.value
                >= Decimal::one()
            {
                return Err(ContractError::InvalidTotalFee {});
            };

            if percentage_payment.address.is_some() {
                deps.api
                    .addr_validate(&percentage_payment.address.clone().unwrap())?;
            };

            PERCENTAGE_FEES.save(deps.storage, (&module_name, &fee_name), &percentage_payment)?;

            event_attributes.push(("value", percentage_payment.value.to_string()));
            if let Some(payment_address) = percentage_payment.address {
                event_attributes.push(("address", payment_address.to_string()));
            }
        }
    }

    Ok(Response::new().add_event(
        Event::new("komple_framework")
            .add_attribute("module", "fee_module")
            .add_attribute("action", "set_fee")
            .add_attribute("fee_type", fee_type.as_str())
            .add_attribute("module_name", &module_name)
            .add_attribute("fee_name", &fee_name)
            .add_attributes(event_attributes),
    ))
}

fn execute_remove_fee(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    fee_type: Fees,
    module_name: String,
    fee_name: String,
) -> Result<Response, ContractError> {
    let hub_addr = HUB_ADDR.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        hub_addr,
        None,
    )?;

    match fee_type {
        Fees::Fixed => FIXED_FEES.remove(deps.storage, (&module_name, &fee_name)),
        Fees::Percentage => PERCENTAGE_FEES.remove(deps.storage, (&module_name, &fee_name)),
    }

    Ok(Response::new().add_event(
        Event::new("komple_framework")
            .add_attribute("module", "fee_module")
            .add_attribute("action", "remove_fee")
            .add_attribute("fee_type", fee_type.as_str())
            .add_attribute("module_name", &module_name)
            .add_attribute("fee_name", &fee_name),
    ))
}

fn execute_distribute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    fee_type: Fees,
    module_name: String,
    custom_payment_addresses: Option<Vec<CustomPaymentAddress>>,
) -> Result<Response, ContractError> {
    let mut msgs: Vec<CosmosMsg> = vec![];

    if info.funds.len() != 1 {
        return Err(FundsError::MissingFunds {}.into());
    };
    let fund = &info.funds[0];

    match fee_type {
        Fees::Fixed => {
            // All of the available amounts to distribute fee
            let amounts = FIXED_FEES
                .prefix(&module_name)
                .range(deps.storage, None, None, Order::Ascending)
                .map(|item| {
                    let (fee_name, fixed_payment) = item.unwrap();
                    FixedFeeResponse {
                        module_name: module_name.clone(),
                        fee_name,
                        address: fixed_payment.address,
                        value: fixed_payment.value,
                    }
                })
                .collect::<Vec<FixedFeeResponse>>();

            if amounts.len() == 0 {
                return Err(ContractError::NoPaymentsFound {});
            }

            // Total amount
            let total_amount = amounts.iter().map(|item| item.value).sum::<Uint128>();
            check_single_amount(&info, total_amount)?;

            // Make bank send messages based on fee percentage
            for amount in amounts {
                let mut is_custom_address = false;

                // If we have some custom addresses find and replace the address
                if let Some(custom_payment_addresses) = custom_payment_addresses.clone() {
                    // Find the correct custom address based on share name
                    let custom_payment_address = custom_payment_addresses
                        .iter()
                        .find(|item| amount.fee_name == item.fee_name);
                    if let Some(custom_payment_address) = custom_payment_address {
                        is_custom_address = true;
                        msgs.push(CosmosMsg::Bank(BankMsg::Send {
                            to_address: custom_payment_address.address.to_string(),
                            amount: vec![coin(amount.value.u128(), fund.denom.clone())],
                        }))
                    }
                }

                // Carry on without replacing the addresses
                if !is_custom_address {
                    if let Some(payment_address) = amount.address {
                        msgs.push(CosmosMsg::Bank(BankMsg::Send {
                            to_address: payment_address.to_string(),
                            amount: vec![coin(amount.value.u128(), fund.denom.clone())],
                        }))
                    };
                }
            }
        }
        Fees::Percentage => {
            // All of the available percentages to distribute fee
            let percentages = PERCENTAGE_FEES
                .prefix(&module_name)
                .range(deps.storage, None, None, Order::Ascending)
                .map(|item| {
                    let (fee_name, percentage_payment) = item.unwrap();
                    PercentageFeeResponse {
                        module_name: module_name.clone(),
                        fee_name,
                        address: percentage_payment.address,
                        value: percentage_payment.value,
                    }
                })
                .collect::<Vec<PercentageFeeResponse>>();

            if percentages.len() == 0 {
                return Err(ContractError::NoPaymentsFound {});
            }

            // Total amount of fee percentage
            let total_fee = percentages
                .iter()
                .map(|item| item.value)
                .sum::<Decimal>()
                .mul(Uint128::new(100));

            // Make bank send messages based on fee percentage
            for percentage in percentages {
                let mut is_custom_address = false;

                // Payment amount is total_funds * percentage / total_fee
                let payment_amount = fund
                    .amount
                    .mul(percentage.value.mul(Uint128::new(100)))
                    .checked_div(total_fee)?;

                // If we have some custom addresses find and replace the address
                if let Some(custom_payment_addresses) = custom_payment_addresses.clone() {
                    // Find the correct custom address based on share name
                    let custom_payment_address = custom_payment_addresses
                        .iter()
                        .find(|item| percentage.fee_name == item.fee_name);
                    if let Some(custom_payment_address) = custom_payment_address {
                        is_custom_address = true;
                        msgs.push(CosmosMsg::Bank(BankMsg::Send {
                            to_address: custom_payment_address.address.to_string(),
                            amount: vec![coin(payment_amount.u128(), fund.denom.clone())],
                        }))
                    }
                }

                // Carry on without replacing the addresses
                if !is_custom_address {
                    if let Some(payment_address) = percentage.address {
                        msgs.push(CosmosMsg::Bank(BankMsg::Send {
                            to_address: payment_address.to_string(),
                            amount: vec![coin(payment_amount.u128(), fund.denom.clone())],
                        }))
                    };
                }
            }
        }
    }

    Ok(Response::new().add_messages(msgs).add_event(
        Event::new("komple_framework")
            .add_attribute("module", Modules::Fee.as_str())
            .add_attribute("action", "distribute")
            .add_attribute("fee_type", fee_type.as_str())
            .add_attribute("module_name", &module_name),
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::PercentageFee {
            module_name,
            fee_name,
        } => to_binary(&query_percentage_fee(deps, module_name, fee_name)?),
        QueryMsg::FixedFee {
            module_name,
            fee_name,
        } => to_binary(&query_fixed_fee(deps, module_name, fee_name)?),
        QueryMsg::PercentageFees {
            module_name,
            start_after,
            limit,
        } => to_binary(&query_percentage_fees(
            deps,
            module_name,
            start_after,
            limit,
        )?),
        QueryMsg::FixedFees {
            module_name,
            start_after,
            limit,
        } => to_binary(&query_fixed_fees(deps, module_name, start_after, limit)?),
        QueryMsg::TotalPercentageFees { module_name } => {
            to_binary(&query_total_percentage_fees(deps, module_name)?)
        }
        QueryMsg::TotalFixedFees { module_name } => {
            to_binary(&query_total_fixed_fees(deps, module_name)?)
        }
        QueryMsg::Modules {
            fee_type,
            // start_after,
            limit,
        } => to_binary(&query_modules(deps, fee_type, limit)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ResponseWrapper<Config>> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ResponseWrapper {
        query: "config".to_string(),
        data: config,
    })
}

fn query_percentage_fee(
    deps: Deps,
    module_name: String,
    fee_name: String,
) -> StdResult<ResponseWrapper<PercentageFeeResponse>> {
    let percentage_fee = PERCENTAGE_FEES.load(deps.storage, (&module_name, &fee_name))?;
    Ok(ResponseWrapper {
        query: "percentage_fee".to_string(),
        data: PercentageFeeResponse {
            module_name,
            fee_name,
            address: percentage_fee.address,
            value: percentage_fee.value,
        },
    })
}

fn query_fixed_fee(
    deps: Deps,
    module_name: String,
    fee_name: String,
) -> StdResult<ResponseWrapper<FixedFeeResponse>> {
    let fixed_fee = FIXED_FEES.load(deps.storage, (&module_name, &fee_name))?;
    Ok(ResponseWrapper {
        query: "fixed_fee".to_string(),
        data: FixedFeeResponse {
            module_name,
            fee_name,
            address: fixed_fee.address,
            value: fixed_fee.value,
        },
    })
}

fn query_percentage_fees(
    deps: Deps,
    module_name: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<ResponseWrapper<Vec<PercentageFeeResponse>>> {
    let limit = limit.unwrap_or(30) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let percentage_fees = PERCENTAGE_FEES
        .prefix(&module_name)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (fee_name, percentage_payment) = item.unwrap();
            PercentageFeeResponse {
                module_name: module_name.clone(),
                fee_name,
                address: percentage_payment.address,
                value: percentage_payment.value,
            }
        })
        .collect::<Vec<PercentageFeeResponse>>();

    Ok(ResponseWrapper {
        query: "percentage_fees".to_string(),
        data: percentage_fees,
    })
}

fn query_fixed_fees(
    deps: Deps,
    module_name: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<ResponseWrapper<Vec<FixedFeeResponse>>> {
    let limit = limit.unwrap_or(30) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let fixed_fees = FIXED_FEES
        .prefix(&module_name)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (fee_name, fixed_payment) = item.unwrap();
            FixedFeeResponse {
                module_name: module_name.clone(),
                fee_name,
                address: fixed_payment.address,
                value: fixed_payment.value,
            }
        })
        .collect::<Vec<FixedFeeResponse>>();

    Ok(ResponseWrapper {
        query: "fixed_fees".to_string(),
        data: fixed_fees,
    })
}

fn query_total_percentage_fees(
    deps: Deps,
    module_name: String,
) -> StdResult<ResponseWrapper<Decimal>> {
    // TODO: Add filters
    let total_percentage = PERCENTAGE_FEES
        .prefix(&module_name)
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (_, percentage_payment) = item.unwrap();
            percentage_payment.value
        })
        .sum::<Decimal>();
    Ok(ResponseWrapper {
        query: "total_percentage_fees".to_string(),
        data: total_percentage,
    })
}

fn query_total_fixed_fees(deps: Deps, module_name: String) -> StdResult<ResponseWrapper<Uint128>> {
    // TODO: Add filters
    let total_fixed = FIXED_FEES
        .prefix(&module_name)
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (_, fixed_payment) = item.unwrap();
            fixed_payment.value
        })
        .sum::<Uint128>();
    Ok(ResponseWrapper {
        query: "total_fixed_fees".to_string(),
        data: total_fixed,
    })
}

fn query_modules(
    deps: Deps,
    fee_type: Fees,
    // start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<ResponseWrapper<Vec<String>>> {
    let limit = limit.unwrap_or(30) as usize;
    // TODO: This does not work
    // None for now but needs fixing
    // let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let modules = match fee_type {
        Fees::Fixed => FIXED_FEES
            .keys(deps.storage, None, None, Order::Descending)
            .take(limit)
            .map(|item| {
                let (module_name, _) = item.unwrap();
                module_name
            })
            .collect::<Vec<String>>(),
        Fees::Percentage => PERCENTAGE_FEES
            .keys(deps.storage, None, None, Order::Descending)
            .take(limit)
            .map(|item| {
                let (module_name, _) = item.unwrap();
                module_name
            })
            .collect::<Vec<String>>(),
    };

    Ok(ResponseWrapper {
        query: "modules".to_string(),
        data: modules,
    })
}
