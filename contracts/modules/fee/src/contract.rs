use std::ops::Mul;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, from_binary, to_binary, Attribute, BankMsg, Binary, CosmosMsg, Decimal, Deps, DepsMut,
    Env, MessageInfo, Order, Response, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw_storage_plus::Bound;
use komple_framework_types::modules::fee::{Fees, FixedPayment, PercentagePayment};
use komple_framework_types::modules::Modules;
use komple_framework_types::shared::query::ResponseWrapper;
use komple_framework_types::shared::RegisterMsg;
use komple_framework_utils::check_admin_privileges;
use komple_framework_utils::funds::{check_single_amount, FundsError};
use komple_framework_utils::response::{EventHelper, ResponseHelper};
use komple_framework_utils::shared::{execute_lock_execute, execute_update_operators};

use crate::error::ContractError;
use crate::msg::{
    CustomPaymentAddress, ExecuteMsg, FixedFeeResponse, PercentageFeeResponse, QueryMsg, ReceiveMsg,
};
use crate::state::{
    Config, CONFIG, EXECUTE_LOCK, FIXED_FEES, HUB_ADDR, OPERATORS, PERCENTAGE_FEES,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:komple-framework-fee-module";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: RegisterMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = deps.api.addr_validate(&msg.admin)?;

    let config = Config { admin };
    CONFIG.save(deps.storage, &config)?;

    HUB_ADDR.save(deps.storage, &info.sender)?;

    EXECUTE_LOCK.save(deps.storage, &false)?;

    Ok(ResponseHelper::new_module("fee", "instantiate").add_event(
        EventHelper::new("fee_instantiate")
            .add_attribute("admin", config.admin.to_string())
            .add_attribute("hub_addr", info.sender)
            .get(),
    ))
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
            None,
        ),
        ExecuteMsg::UpdateOperators { addrs } => {
            let config = CONFIG.load(deps.storage)?;
            let res = execute_update_operators(
                deps,
                info,
                Modules::Fee.as_str(),
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
                Modules::Fee.as_str(),
                &env.contract.address,
                EXECUTE_LOCK,
            );
            match res {
                Ok(res) => Ok(res),
                Err(e) => Err(e.into()),
            }
        }
        ExecuteMsg::Receive(msg) => execute_receive(deps, env, info, msg),
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
    let operators = OPERATORS.may_load(deps.storage)?;
    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        hub_addr,
        operators,
    )?;

    let mut event_attributes: Vec<Attribute> = vec![];

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

            event_attributes.push(Attribute {
                key: "value".to_string(),
                value: fixed_payment.value.to_string(),
            });
            if let Some(payment_address) = fixed_payment.address {
                event_attributes.push(Attribute {
                    key: "address".to_string(),
                    value: payment_address,
                });
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
            let total_fee =
                query_total_percentage_fees(deps.as_ref(), module_name.clone(), None, None)?;
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

            event_attributes.push(Attribute {
                key: "value".to_string(),
                value: percentage_payment.value.to_string(),
            });
            if let Some(payment_address) = percentage_payment.address {
                event_attributes.push(Attribute {
                    key: "address".to_string(),
                    value: payment_address,
                });
            }
        }
    }

    Ok(ResponseHelper::new_module("fee", "set_fee").add_event(
        EventHelper::new("fee_set_fee")
            .add_attribute("fee_type", fee_type.as_str())
            .add_attribute("module_name", &module_name)
            .add_attribute("fee_name", &fee_name)
            .add_attributes(event_attributes)
            .get(),
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
    let operators = OPERATORS.may_load(deps.storage)?;
    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        hub_addr,
        operators,
    )?;

    match fee_type {
        Fees::Fixed => FIXED_FEES.remove(deps.storage, (&module_name, &fee_name)),
        Fees::Percentage => PERCENTAGE_FEES.remove(deps.storage, (&module_name, &fee_name)),
    }

    Ok(ResponseHelper::new_module("fee", "remove_fee").add_event(
        EventHelper::new("fee_remove_fee")
            .add_attribute("fee_type", fee_type.as_str())
            .add_attribute("module_name", &module_name)
            .add_attribute("fee_name", &fee_name)
            .get(),
    ))
}

fn execute_distribute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    fee_type: Fees,
    module_name: String,
    custom_payment_addresses: Option<Vec<CustomPaymentAddress>>,
    cw20_token_amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let msgs = match fee_type {
        Fees::Fixed => _distribute_fixed_fee(
            deps,
            info,
            &module_name,
            custom_payment_addresses,
            cw20_token_amount,
        )?,
        Fees::Percentage => _distribute_percentage_fee(
            deps,
            info,
            &module_name,
            custom_payment_addresses,
            cw20_token_amount,
        )?,
    };

    Ok(ResponseHelper::new_module("fee", "distribute")
        .add_messages(msgs)
        .add_event(
            EventHelper::new("fee_distribute")
                .add_attribute("fee_type", fee_type.as_str())
                .add_attribute("module_name", &module_name)
                .get(),
        ))
}

fn _distribute_fixed_fee(
    deps: DepsMut,
    info: MessageInfo,
    module_name: &str,
    custom_payment_addresses: Option<Vec<CustomPaymentAddress>>,
    cw20_token_amount: Option<Uint128>,
) -> Result<Vec<CosmosMsg>, ContractError> {
    let mut msgs: Vec<CosmosMsg> = vec![];

    // All of the available amounts to distribute fee
    let amounts = FIXED_FEES
        .prefix(&module_name)
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (fee_name, fixed_payment) = item.unwrap();
            FixedFeeResponse {
                module_name: module_name.to_string(),
                fee_name,
                address: fixed_payment.address,
                value: fixed_payment.value,
            }
        })
        .collect::<Vec<FixedFeeResponse>>();

    if amounts.is_empty() {
        return Err(ContractError::NoPaymentsFound {});
    }

    // Total amount
    let total_amount = amounts.iter().map(|item| item.value).sum::<Uint128>();
    if cw20_token_amount.is_none() {
        check_single_amount(&info, total_amount)?;
    } else {
        if cw20_token_amount.unwrap() != total_amount {
            return Err(FundsError::InvalidFunds {
                got: cw20_token_amount.unwrap().to_string(),
                expected: total_amount.to_string(),
            }
            .into());
        };
    }

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
                let msg = match cw20_token_amount.is_none() {
                    true => CosmosMsg::Bank(BankMsg::Send {
                        to_address: custom_payment_address.address.to_string(),
                        amount: vec![coin(amount.value.u128(), info.funds[0].denom.clone())],
                    }),
                    false => CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: info.sender.to_string(),
                        msg: to_binary(&Cw20ExecuteMsg::Transfer {
                            recipient: custom_payment_address.address.to_string(),
                            amount: amount.value,
                        })?,
                        funds: vec![],
                    }),
                };
                msgs.push(msg);
            }
        }

        // Carry on without replacing the addresses
        if !is_custom_address {
            if let Some(payment_address) = amount.address {
                let msg = match cw20_token_amount.is_none() {
                    true => CosmosMsg::Bank(BankMsg::Send {
                        to_address: payment_address.to_string(),
                        amount: vec![coin(amount.value.u128(), info.funds[0].denom.clone())],
                    }),
                    false => CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: info.sender.to_string(),
                        msg: to_binary(&Cw20ExecuteMsg::Transfer {
                            recipient: payment_address.to_string(),
                            amount: amount.value,
                        })?,
                        funds: vec![],
                    }),
                };
                msgs.push(msg);
            };
        }
    }

    Ok(msgs)
}

fn _distribute_percentage_fee(
    deps: DepsMut,
    info: MessageInfo,
    module_name: &str,
    custom_payment_addresses: Option<Vec<CustomPaymentAddress>>,
    cw20_token_amount: Option<Uint128>,
) -> Result<Vec<CosmosMsg>, ContractError> {
    let mut msgs: Vec<CosmosMsg> = vec![];

    // All of the available percentages to distribute fee
    let percentages = PERCENTAGE_FEES
        .prefix(&module_name)
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (fee_name, percentage_payment) = item.unwrap();
            PercentageFeeResponse {
                module_name: module_name.to_string(),
                fee_name,
                address: percentage_payment.address,
                value: percentage_payment.value,
            }
        })
        .collect::<Vec<PercentageFeeResponse>>();

    if percentages.is_empty() {
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
        let payment_amount = match cw20_token_amount {
            Some(amount) => amount
                .mul(percentage.value.mul(Uint128::new(100)))
                .checked_div(total_fee)?,
            None => info.funds[0]
                .amount
                .mul(percentage.value.mul(Uint128::new(100)))
                .checked_div(total_fee)?,
        };

        // If we have some custom addresses find and replace the address
        if let Some(custom_payment_addresses) = custom_payment_addresses.clone() {
            // Find the correct custom address based on share name
            let custom_payment_address = custom_payment_addresses
                .iter()
                .find(|item| percentage.fee_name == item.fee_name);
            if let Some(custom_payment_address) = custom_payment_address {
                is_custom_address = true;
                let msg = match cw20_token_amount.is_none() {
                    true => CosmosMsg::Bank(BankMsg::Send {
                        to_address: custom_payment_address.address.to_string(),
                        amount: vec![coin(payment_amount.u128(), info.funds[0].denom.clone())],
                    }),
                    false => CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: info.sender.to_string(),
                        msg: to_binary(&Cw20ExecuteMsg::Transfer {
                            recipient: custom_payment_address.address.to_string(),
                            amount: payment_amount,
                        })?,
                        funds: vec![],
                    }),
                };
                msgs.push(msg);
            }
        }

        // Carry on without replacing the addresses
        if !is_custom_address {
            if let Some(payment_address) = percentage.address {
                let msg = match cw20_token_amount.is_none() {
                    true => CosmosMsg::Bank(BankMsg::Send {
                        to_address: payment_address.to_string(),
                        amount: vec![coin(payment_amount.u128(), info.funds[0].denom.clone())],
                    }),
                    false => CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: info.sender.to_string(),
                        msg: to_binary(&Cw20ExecuteMsg::Transfer {
                            recipient: payment_address.to_string(),
                            amount: payment_amount,
                        })?,
                        funds: vec![],
                    }),
                };
                msgs.push(msg);
            };
        }
    }

    Ok(msgs)
}

fn execute_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_receive_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: ReceiveMsg = from_binary(&cw20_receive_msg.msg)?;
    let amount = cw20_receive_msg.amount;

    match msg {
        ReceiveMsg::Distribute {
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
            Some(amount),
        ),
    }
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
        QueryMsg::TotalPercentageFees {
            module_name,
            start_after,
            limit,
        } => to_binary(&query_total_percentage_fees(
            deps,
            module_name,
            start_after,
            limit,
        )?),
        QueryMsg::TotalFixedFees {
            module_name,
            start_after,
            limit,
        } => to_binary(&query_total_fixed_fees(
            deps,
            module_name,
            start_after,
            limit,
        )?),
        QueryMsg::Keys {
            fee_type,
            start_after,
            limit,
        } => to_binary(&query_keys(deps, fee_type, start_after, limit)?),
        QueryMsg::Operators {} => to_binary(&query_operators(deps)?),
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
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<ResponseWrapper<Decimal>> {
    let limit = limit.unwrap_or(30) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let total_percentage = PERCENTAGE_FEES
        .prefix(&module_name)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
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

fn query_total_fixed_fees(
    deps: Deps,
    module_name: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<ResponseWrapper<Uint128>> {
    let limit = limit.unwrap_or(30) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let total_fixed = FIXED_FEES
        .prefix(&module_name)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
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

// TODO: Fix this query
// Need to add pagination
fn query_keys(
    deps: Deps,
    fee_type: Fees,
    _start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<ResponseWrapper<Vec<String>>> {
    let limit = limit.unwrap_or(30) as usize;
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

fn query_operators(deps: Deps) -> StdResult<ResponseWrapper<Vec<String>>> {
    let addrs = OPERATORS.may_load(deps.storage)?;
    let addrs = match addrs {
        Some(addrs) => addrs.iter().map(|a| a.to_string()).collect(),
        None => vec![],
    };
    Ok(ResponseWrapper::new("operators", addrs))
}
