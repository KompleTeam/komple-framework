#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, Timestamp,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::maybe_addr;
use rift_types::query::ResponseWrapper;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, WhitelistConfig, CONFIG, WHITELIST, WHITELIST_CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:rift-whitelist-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if msg.member_limit == 0 {
        return Err(ContractError::InvalidMemberLimit {});
    }

    if msg.per_address_limit == 0 {
        return Err(ContractError::InvalidPerAddressLimit {});
    }

    if msg.members.len() == 0 {
        return Err(ContractError::EmptyMemberList {});
    }

    if msg.start_time <= env.block.time {
        return Err(ContractError::InvalidStartTime {});
    }
    if msg.end_time < env.block.time {
        return Err(ContractError::InvalidEndTime {});
    }
    if msg.start_time >= msg.end_time {
        return Err(ContractError::InvalidStartTime {});
    }

    let config = Config { admin: info.sender };
    CONFIG.save(deps.storage, &config)?;

    msg.members.sort_unstable();
    msg.members.dedup();

    let member_num = msg.members.len() as u16;

    for member in msg.members.into_iter() {
        let addr = deps.api.addr_validate(&member.clone())?;
        WHITELIST.save(deps.storage, addr, &true)?;
    }

    let whitelist_config = WhitelistConfig {
        start_time: msg.start_time,
        end_time: msg.end_time,
        unit_price: msg.unit_price,
        per_address_limit: msg.per_address_limit,
        member_limit: msg.member_limit,
        member_num,
    };
    WHITELIST_CONFIG.save(deps.storage, &whitelist_config)?;

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
        ExecuteMsg::UpdateStartTime(start_time) => {
            execute_update_start_time(deps, env, info, start_time)
        }
        ExecuteMsg::UpdateEndTime(end_time) => execute_update_end_time(deps, env, info, end_time),
        ExecuteMsg::AddMembers(members) => execute_add_members(deps, env, info, members),
        ExecuteMsg::RemoveMembers(members) => execute_remove_members(deps, env, info, members),
        ExecuteMsg::UpdatePerAddressLimit(limit) => {
            execute_update_per_address_limit(deps, env, info, limit)
        }
        ExecuteMsg::UpdateMemberLimit(limit) => execute_update_member_limit(deps, env, info, limit),
    }
}

fn execute_update_start_time(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    start_time: Timestamp,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let mut whitelist_config = WHITELIST_CONFIG.load(deps.storage)?;

    if env.block.time >= whitelist_config.start_time {
        return Err(ContractError::AlreadyStarted {});
    }
    if start_time <= env.block.time {
        return Err(ContractError::InvalidStartTime {});
    }
    if start_time >= whitelist_config.end_time {
        return Err(ContractError::InvalidStartTime {});
    }

    whitelist_config.start_time = start_time;
    WHITELIST_CONFIG.save(deps.storage, &whitelist_config)?;

    Ok(Response::new().add_attribute("action", "execute_update_start_time"))
}

fn execute_update_end_time(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    end_time: Timestamp,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let mut whitelist_config = WHITELIST_CONFIG.load(deps.storage)?;

    if env.block.time >= whitelist_config.start_time {
        return Err(ContractError::AlreadyStarted {});
    }
    if end_time <= env.block.time {
        return Err(ContractError::InvalidEndTime {});
    }
    if end_time <= whitelist_config.start_time {
        return Err(ContractError::InvalidEndTime {});
    }

    whitelist_config.end_time = end_time;
    WHITELIST_CONFIG.save(deps.storage, &whitelist_config)?;

    Ok(Response::new().add_attribute("action", "execute_update_end_time"))
}

fn execute_add_members(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut members: Vec<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let mut whitelist_config = WHITELIST_CONFIG.load(deps.storage)?;
    if env.block.time >= whitelist_config.start_time {
        return Err(ContractError::AlreadyStarted {});
    }

    members.sort_unstable();
    members.dedup();

    for member in members {
        if whitelist_config.member_num >= whitelist_config.member_limit {
            return Err(ContractError::MemberLimitExceeded {});
        }
        let addr = deps.api.addr_validate(&member)?;
        if WHITELIST.has(deps.storage, addr.clone()) {
            return Err(ContractError::MemberExists {});
        }
        WHITELIST.save(deps.storage, addr, &true)?;
        whitelist_config.member_num += 1;
    }

    WHITELIST_CONFIG.save(deps.storage, &whitelist_config)?;

    Ok(Response::new().add_attribute("action", "execute_add_members"))
}

fn execute_remove_members(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut members: Vec<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let mut whitelist_config = WHITELIST_CONFIG.load(deps.storage)?;
    if env.block.time >= whitelist_config.start_time {
        return Err(ContractError::AlreadyStarted {});
    }

    members.sort_unstable();
    members.dedup();

    for member in members {
        let addr = deps.api.addr_validate(&member)?;
        if !WHITELIST.has(deps.storage, addr.clone()) {
            return Err(ContractError::MemberNotFound {});
        }
        WHITELIST.remove(deps.storage, addr);
        whitelist_config.member_num -= 1;
    }

    WHITELIST_CONFIG.save(deps.storage, &whitelist_config)?;

    Ok(Response::new().add_attribute("action", "execute_add_members"))
}

fn execute_update_per_address_limit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    limit: u8,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let mut whitelist_config = WHITELIST_CONFIG.load(deps.storage)?;
    if env.block.time >= whitelist_config.start_time {
        return Err(ContractError::AlreadyStarted {});
    }

    if limit == 0 {
        return Err(ContractError::InvalidPerAddressLimit {});
    }

    whitelist_config.per_address_limit = limit;
    WHITELIST_CONFIG.save(deps.storage, &whitelist_config)?;

    Ok(Response::new().add_attribute("action", "execute_update_per_address_limit"))
}

fn execute_update_member_limit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    limit: u16,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let mut whitelist_config = WHITELIST_CONFIG.load(deps.storage)?;
    if env.block.time >= whitelist_config.start_time {
        return Err(ContractError::AlreadyStarted {});
    }
    if limit == 0 {
        return Err(ContractError::InvalidMemberLimit {});
    }

    whitelist_config.member_limit = limit;
    WHITELIST_CONFIG.save(deps.storage, &whitelist_config)?;

    Ok(Response::new().add_attribute("action", "execute_update_member_limit"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps, env)?),
        QueryMsg::HasStarted {} => to_binary(&query_has_started(deps, env)?),
        QueryMsg::HasEnded {} => to_binary(&query_has_ended(deps, env)?),
        QueryMsg::IsActive {} => to_binary(&query_is_active(deps, env)?),
        QueryMsg::Members { start_after, limit } => {
            to_binary(&query_members(deps, start_after, limit)?)
        }
        QueryMsg::HasMember { member } => to_binary(&query_has_member(deps, member)?),
    }
}

fn query_config(deps: Deps, env: Env) -> StdResult<ResponseWrapper<ConfigResponse>> {
    let config = CONFIG.load(deps.storage)?;
    let whitelist_config = WHITELIST_CONFIG.load(deps.storage)?;
    let config_res = ConfigResponse {
        admin: config.admin.to_string(),
        start_time: whitelist_config.start_time,
        end_time: whitelist_config.end_time,
        unit_price: whitelist_config.unit_price,
        per_address_limit: whitelist_config.per_address_limit,
        member_limit: whitelist_config.member_limit,
        member_num: whitelist_config.member_num,
        is_active: get_active_status(deps, env)?,
    };
    Ok(ResponseWrapper::new("config", config_res))
}

fn query_has_started(deps: Deps, env: Env) -> StdResult<ResponseWrapper<bool>> {
    let whitelist_config = WHITELIST_CONFIG.load(deps.storage)?;
    Ok(ResponseWrapper::new(
        "has_started",
        env.block.time >= whitelist_config.start_time,
    ))
}

fn query_has_ended(deps: Deps, env: Env) -> StdResult<ResponseWrapper<bool>> {
    let whitelist_config = WHITELIST_CONFIG.load(deps.storage)?;
    Ok(ResponseWrapper::new(
        "has_end",
        env.block.time >= whitelist_config.end_time,
    ))
}

fn query_is_active(deps: Deps, env: Env) -> StdResult<ResponseWrapper<bool>> {
    Ok(ResponseWrapper::new(
        "is_active",
        get_active_status(deps, env)?,
    ))
}

fn query_members(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u8>,
) -> StdResult<ResponseWrapper<Vec<String>>> {
    let limit = limit.unwrap_or(10) as usize;
    let start_addr = maybe_addr(deps.api, start_after)?;
    let start = start_addr.map(Bound::exclusive);
    let members = WHITELIST
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|addr| addr.unwrap().0.to_string())
        .collect::<Vec<String>>();
    Ok(ResponseWrapper::new("members", members))
}

fn query_has_member(deps: Deps, member: String) -> StdResult<ResponseWrapper<bool>> {
    let addr = deps.api.addr_validate(&member)?;
    let exists = WHITELIST.has(deps.storage, addr);
    Ok(ResponseWrapper::new("has_member", exists))
}

fn get_active_status(deps: Deps, env: Env) -> StdResult<bool> {
    let whitelist_config = WHITELIST_CONFIG.load(deps.storage)?;
    Ok(env.block.time >= whitelist_config.start_time && env.block.time < whitelist_config.end_time)
}
