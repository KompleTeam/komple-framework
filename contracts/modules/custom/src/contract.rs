#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use komple_framework_types::shared::query::ResponseWrapper;
use semver::Version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG, EXECUTE_LOCK, HUB_ADDR, OPERATORS};

use komple_framework_types::shared::RegisterMsg;
use komple_framework_utils::{
    response::{EventHelper, ResponseHelper},
    shared::{execute_lock_execute, execute_update_operators},
};

// version info for migration info
/* TODO: Change module name here */
const CONTRACT_NAME: &str = "crates.io:komple-framework-custom-module";
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

    Ok(
        /* TODO: Change module name here */
        ResponseHelper::new_module("custom_name", "instantiate").add_event(
            /* TODO: Change module name here */
            EventHelper::new("custom_name_instantiate")
                .add_attribute("admin", config.admin)
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
        /* TODO: Add execute messages here */
        /* ... */
        ExecuteMsg::UpdateOperators { addrs } => {
            let config = CONFIG.load(deps.storage)?;
            let res = execute_update_operators(
                deps,
                info,
                /* TODO: Change module name here */
                "custom_name",
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
            let res =
                /* TODO: Change module name here */
                execute_lock_execute(deps, info, "custom_name", &env.contract.address, EXECUTE_LOCK);
            match res {
                Ok(res) => Ok(res),
                Err(err) => Err(err.into()),
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        /* TODO: Add query messages here */
        /* ... */
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Operators {} => to_binary(&query_operators(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ResponseWrapper<Config>> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ResponseWrapper::new("config", config))
}

fn query_operators(deps: Deps) -> StdResult<ResponseWrapper<Vec<String>>> {
    let addrs = OPERATORS.load(deps.storage)?;
    Ok(ResponseWrapper::new(
        "operators",
        addrs.iter().map(|addr| addr.to_string()).collect(),
    ))
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
