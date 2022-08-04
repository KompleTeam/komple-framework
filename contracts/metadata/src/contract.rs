#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use rift_utils::check_admin_privileges;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, LockResponse, MetadataResponse, QueryMsg};
use crate::state::{
    Config, Metadata, Trait, ATTRIBUTES, COLLECTION_ADDR, CONFIG, METADATA, METADATA_LOCK,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:minter";
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
        update_lock: false,
    };

    CONFIG.save(deps.storage, &config)?;

    COLLECTION_ADDR.save(deps.storage, &info.sender)?;

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
        ExecuteMsg::AddMetadata {
            token_id,
            metadata,
            attributes,
        } => execute_add_metadata(deps, env, info, token_id, metadata, attributes),
        ExecuteMsg::UpdateMetadata { token_id, metadata } => {
            execute_update_metadata(deps, env, info, token_id, metadata)
        }
        ExecuteMsg::AddAttribute {
            token_id,
            attribute,
        } => execute_add_attribute(deps, env, info, token_id, attribute),
        ExecuteMsg::UpdateAttribute {
            token_id,
            attribute,
        } => execute_update_attribute(deps, env, info, token_id, attribute),
        ExecuteMsg::RemoveAttribute {
            token_id,
            trait_type,
        } => execute_remove_attribute(deps, env, info, token_id, trait_type),
    }
}

fn execute_add_metadata(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    metadata: Metadata,
    attribute: Vec<Trait>,
) -> Result<Response, ContractError> {
    let collection_addr = COLLECTION_ADDR.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        collection_addr,
        None,
    )?;

    METADATA.save(deps.storage, &token_id, &metadata)?;
    for attribute in attribute {
        ATTRIBUTES.save(
            deps.storage,
            (&token_id, &attribute.trait_type),
            &attribute.value,
        )?;
    }

    Ok(Response::new().add_attribute("action", "execute_add_metadata"))
}

fn execute_update_metadata(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    metadata: Metadata,
) -> Result<Response, ContractError> {
    let collection_addr = COLLECTION_ADDR.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        collection_addr,
        None,
    )?;
    check_metadata_lock(&deps, &config, &token_id)?;

    METADATA.save(deps.storage, &token_id, &metadata)?;

    Ok(Response::new().add_attribute("action", "execute_update_metadata"))
}

fn execute_add_attribute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    attribute: Trait,
) -> Result<Response, ContractError> {
    let collection_addr = COLLECTION_ADDR.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        collection_addr,
        None,
    )?;
    check_metadata_lock(&deps, &config, &token_id)?;

    let value = ATTRIBUTES.may_load(deps.storage, (&token_id, &attribute.trait_type))?;
    if value.is_some() {
        return Err(ContractError::AttributeAlreadyExists {});
    }

    ATTRIBUTES.save(
        deps.storage,
        (&token_id, &attribute.trait_type),
        &attribute.value,
    )?;

    Ok(Response::new().add_attribute("action", "execute_add_attribute"))
}

fn execute_update_attribute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    attribute: Trait,
) -> Result<Response, ContractError> {
    let collection_addr = COLLECTION_ADDR.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        collection_addr,
        None,
    )?;
    check_metadata_lock(&deps, &config, &token_id)?;

    let value = ATTRIBUTES.may_load(deps.storage, (&token_id, &attribute.trait_type))?;
    if value.is_none() {
        return Err(ContractError::AttributeNotFound {});
    }

    ATTRIBUTES.save(
        deps.storage,
        (&token_id, &attribute.trait_type),
        &attribute.value,
    )?;

    Ok(Response::new().add_attribute("action", "execute_update_attribute"))
}

fn execute_remove_attribute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    trait_type: String,
) -> Result<Response, ContractError> {
    let collection_addr = COLLECTION_ADDR.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        collection_addr,
        None,
    )?;
    check_metadata_lock(&deps, &config, &token_id)?;

    let value = ATTRIBUTES.may_load(deps.storage, (&token_id, &trait_type))?;
    if value.is_none() {
        return Err(ContractError::AttributeNotFound {});
    }

    ATTRIBUTES.remove(deps.storage, (&token_id, &trait_type));

    Ok(Response::new().add_attribute("action", "execute_remove_attribute"))
}

fn check_metadata_lock(
    deps: &DepsMut,
    config: &Config,
    token_id: &str,
) -> Result<(), ContractError> {
    let metadata_lock = METADATA_LOCK.may_load(deps.storage, token_id)?;
    if config.update_lock || (metadata_lock.is_some() && metadata_lock.unwrap()) {
        return Err(ContractError::UpdateLocked {});
    }
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Metadata { token_id } => to_binary(&query_metadata(deps, token_id)?),
        QueryMsg::MetadataLock { token_id } => to_binary(&query_metadata_lock(deps, token_id)?),
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

fn query_metadata(deps: Deps, token_id: String) -> StdResult<MetadataResponse> {
    let metadata = METADATA.load(deps.storage, &token_id)?;
    let attributes = ATTRIBUTES
        .prefix(&token_id)
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| -> Trait {
            let r#trait = item.unwrap();
            Trait {
                trait_type: r#trait.0,
                value: r#trait.1,
            }
        })
        .collect::<Vec<Trait>>();
    Ok(MetadataResponse {
        metadata,
        attributes,
    })
}

fn query_metadata_lock(deps: Deps, token_id: String) -> StdResult<LockResponse> {
    let locked = METADATA_LOCK.load(deps.storage, &token_id)?;
    Ok(LockResponse { locked })
}
