#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Attribute, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdError, StdResult,
};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use cw_storage_plus::Bound;
use cw_utils::maybe_addr;
use komple_framework_types::shared::query::ResponseWrapper;
use komple_framework_types::shared::RegisterMsg;
use komple_framework_utils::response::{EventHelper, ResponseHelper};
use semver::Version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, WhitelistConfig, CONFIG, WHITELIST};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:komple-framework-whitelist-module";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: RegisterMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if msg.data.is_none() {
        return Err(ContractError::InvalidInstantiateMsg {});
    };
    let mut data: InstantiateMsg = from_binary(&msg.data.unwrap())?;

    if data.config.member_limit == 0 {
        return Err(ContractError::InvalidMemberLimit {});
    }

    if data.config.per_address_limit == 0 {
        return Err(ContractError::InvalidPerAddressLimit {});
    }

    if data.members.is_empty() {
        return Err(ContractError::EmptyMemberList {});
    }

    if data.config.start_time <= env.block.time {
        return Err(ContractError::InvalidStartTime {});
    }
    if data.config.end_time < env.block.time {
        return Err(ContractError::InvalidEndTime {});
    }
    if data.config.start_time >= data.config.end_time {
        return Err(ContractError::InvalidStartTime {});
    }

    data.members.sort_unstable();
    data.members.dedup();

    let mut event_attributes: Vec<Attribute> = vec![];

    let member_num = data.members.len() as u16;
    let admin = deps.api.addr_validate(&msg.admin)?;
    let config = Config {
        admin,
        start_time: data.config.start_time,
        end_time: data.config.end_time,
        per_address_limit: data.config.per_address_limit,
        member_limit: data.config.member_limit,
        member_num,
    };
    CONFIG.save(deps.storage, &config)?;

    for member in data.members.into_iter() {
        let addr = deps.api.addr_validate(&member.clone())?;
        WHITELIST.save(deps.storage, addr, &true)?;
        event_attributes.push(Attribute {
            key: "member".to_string(),
            value: member,
        });
    }

    Ok(
        ResponseHelper::new_module("whitelist", "instantiate").add_event(
            EventHelper::new("whitelist_instantiate")
                .add_attribute("start_time", config.start_time.to_string())
                .add_attribute("end_time", config.end_time.to_string())
                .add_attribute("per_address_limit", config.per_address_limit.to_string())
                .add_attribute("member_limit", config.member_limit.to_string())
                .add_attributes(event_attributes)
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
    match msg {
        ExecuteMsg::AddMembers { members } => execute_add_members(deps, env, info, members),
        ExecuteMsg::RemoveMembers { members } => execute_remove_members(deps, env, info, members),
        ExecuteMsg::UpdateWhitelistConfig { whitelist_config } => {
            execute_update_whitelist_config(deps, env, info, whitelist_config)
        }
    }
}

fn execute_update_whitelist_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    whitelist_config: WhitelistConfig,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Start time block
    if config.start_time != whitelist_config.start_time {
        if env.block.time >= config.start_time {
            return Err(ContractError::AlreadyStarted {});
        }
        if whitelist_config.start_time <= env.block.time {
            return Err(ContractError::InvalidStartTime {});
        }
        if whitelist_config.start_time >= config.end_time {
            return Err(ContractError::InvalidStartTime {});
        }
        config.start_time = whitelist_config.start_time;
    }

    // End time block
    if config.end_time != whitelist_config.end_time {
        if env.block.time >= config.start_time {
            return Err(ContractError::AlreadyStarted {});
        }
        if whitelist_config.end_time <= env.block.time {
            return Err(ContractError::InvalidEndTime {});
        }
        if whitelist_config.end_time <= config.start_time {
            return Err(ContractError::InvalidEndTime {});
        }
        config.end_time = whitelist_config.end_time;
    }

    // Per address limit block
    if config.per_address_limit != whitelist_config.per_address_limit {
        if env.block.time >= config.start_time {
            return Err(ContractError::AlreadyStarted {});
        }
        if whitelist_config.per_address_limit == 0 {
            return Err(ContractError::InvalidPerAddressLimit {});
        }
        config.per_address_limit = whitelist_config.per_address_limit;
    }

    // Member limit block
    if config.member_limit != whitelist_config.member_limit {
        if env.block.time >= config.start_time {
            return Err(ContractError::AlreadyStarted {});
        }
        if whitelist_config.member_limit == 0 {
            return Err(ContractError::InvalidMemberLimit {});
        }
        config.member_limit = whitelist_config.member_limit;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(
        ResponseHelper::new_module("whitelist", "update_whitelist_config").add_event(
            EventHelper::new("whitelist_update_whitelist_config")
                .add_attribute("start_time", config.start_time.to_string())
                .add_attribute("end_time", config.end_time.to_string())
                .add_attribute("per_address_limit", config.per_address_limit.to_string())
                .add_attribute("member_limit", config.member_limit.to_string())
                .get(),
        ),
    )
}

fn execute_add_members(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut members: Vec<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if env.block.time >= config.start_time {
        return Err(ContractError::AlreadyStarted {});
    }

    members.sort_unstable();
    members.dedup();

    let mut event_attributes: Vec<Attribute> = vec![];

    for member in members {
        if config.member_num >= config.member_limit {
            return Err(ContractError::MemberLimitExceeded {});
        }
        let addr = deps.api.addr_validate(&member)?;
        if WHITELIST.has(deps.storage, addr.clone()) {
            return Err(ContractError::MemberExists {});
        }
        WHITELIST.save(deps.storage, addr, &true)?;
        config.member_num += 1;

        event_attributes.push(Attribute {
            key: "member".to_string(),
            value: member,
        });
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(
        ResponseHelper::new_module("whitelist", "add_members").add_event(
            EventHelper::new("whitelist_add_members")
                .add_attributes(event_attributes)
                .get(),
        ),
    )
}

fn execute_remove_members(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut members: Vec<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if env.block.time >= config.start_time {
        return Err(ContractError::AlreadyStarted {});
    }

    members.sort_unstable();
    members.dedup();

    let mut event_attributes: Vec<Attribute> = vec![];

    for member in members {
        let addr = deps.api.addr_validate(&member)?;
        if !WHITELIST.has(deps.storage, addr.clone()) {
            return Err(ContractError::MemberNotFound {});
        }
        WHITELIST.remove(deps.storage, addr);
        config.member_num -= 1;

        event_attributes.push(Attribute {
            key: "member".to_string(),
            value: member,
        });
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(
        ResponseHelper::new_module("whitelist", "remove_members").add_event(
            EventHelper::new("whitelist_remove_members")
                .add_attributes(event_attributes)
                .get(),
        ),
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps, env)?),
        QueryMsg::IsActive {} => to_binary(&query_is_active(deps, env)?),
        QueryMsg::Members { start_after, limit } => {
            to_binary(&query_members(deps, start_after, limit)?)
        }
        QueryMsg::IsMember { member } => to_binary(&query_is_member(deps, member)?),
    }
}

fn query_config(deps: Deps, env: Env) -> StdResult<ResponseWrapper<ConfigResponse>> {
    let config = CONFIG.load(deps.storage)?;
    let config_res = ConfigResponse {
        admin: config.admin.to_string(),
        start_time: config.start_time,
        end_time: config.end_time,
        per_address_limit: config.per_address_limit,
        member_limit: config.member_limit,
        member_num: config.member_num,
        is_active: get_active_status(deps, env)?,
    };
    Ok(ResponseWrapper::new("config", config_res))
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

fn query_is_member(deps: Deps, member: String) -> StdResult<ResponseWrapper<bool>> {
    let addr = deps.api.addr_validate(&member)?;
    let exists = WHITELIST.has(deps.storage, addr);
    Ok(ResponseWrapper::new("is_member", exists))
}

fn get_active_status(deps: Deps, env: Env) -> StdResult<bool> {
    let config = CONFIG.load(deps.storage)?;
    Ok(env.block.time >= config.start_time && env.block.time < config.end_time)
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
