#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use komple_types::metadata::Metadata as MetadataType;
use komple_types::query::ResponseWrapper;
use komple_utils::check_admin_privileges;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MetadataResponse, QueryMsg};
use crate::state::{
    Config, MetaInfo, Metadata, Trait, COLLECTION_ADDR, CONFIG, DYNAMIC_METADATA, METADATA,
    METADATA_ID, STATIC_METADATA,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:komple-metadata-contract";
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
        metadata_type: msg.metadata_type,
    };

    CONFIG.save(deps.storage, &config)?;

    COLLECTION_ADDR.save(deps.storage, &info.sender)?;

    METADATA_ID.save(deps.storage, &0)?;

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
            meta_info,
            attributes,
        } => execute_add_metadata(deps, env, info, meta_info, attributes),
        ExecuteMsg::LinkMetadata {
            token_id,
            metadata_id,
        } => execute_link_metadata(deps, env, info, token_id, metadata_id),
        ExecuteMsg::UpdateMetaInfo {
            token_id,
            meta_info,
        } => execute_update_meta_info(deps, env, info, token_id, meta_info),
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
    meta_info: MetaInfo,
    attributes: Vec<Trait>,
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

    let metadata = Metadata {
        meta_info,
        attributes,
    };

    let metadata_id = (METADATA_ID.load(deps.storage)?) + 1;

    METADATA.save(deps.storage, metadata_id, &metadata)?;
    METADATA_ID.save(deps.storage, &metadata_id)?;

    Ok(Response::new().add_attribute("action", "execute_add_metadata"))
}

fn execute_link_metadata(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: u32,
    metadata_id: Option<u32>,
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

    let metadata_id = match config.metadata_type {
        MetadataType::OneToOne => token_id,
        MetadataType::Static | MetadataType::Dynamic => {
            if metadata_id.is_none() {
                return Err(ContractError::MissingMetadata {});
            }
            metadata_id.unwrap()
        }
    };

    let metadata = METADATA.may_load(deps.storage, metadata_id)?;
    if metadata.is_none() {
        return Err(ContractError::MissingMetadata {});
    }

    match config.metadata_type {
        MetadataType::OneToOne | MetadataType::Static => {
            STATIC_METADATA.save(deps.storage, token_id, &metadata_id)?;
        }
        MetadataType::Dynamic => {
            let metadata = metadata.unwrap();
            DYNAMIC_METADATA.save(deps.storage, token_id, &metadata)?;
        }
    };

    Ok(Response::new().add_attribute("action", "execute_link_metadata"))
}

fn execute_update_meta_info(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: u32,
    meta_info: MetaInfo,
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
    // check_metadata_lock(&deps, &config, &token_id)?;

    let (metadata_id, mut metadata) =
        get_metadata_from_type(&deps, &config.metadata_type, token_id)?;

    match config.metadata_type {
        MetadataType::OneToOne | MetadataType::Static => {
            metadata.meta_info = meta_info;
            METADATA.save(deps.storage, metadata_id, &metadata)?;
        }
        MetadataType::Dynamic => {
            metadata.meta_info = meta_info;
            DYNAMIC_METADATA.save(deps.storage, token_id, &metadata)?;
        }
    };

    Ok(Response::new().add_attribute("action", "execute_update_meta_info"))
}

fn execute_add_attribute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: u32,
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

    let (metadata_id, mut metadata) =
        get_metadata_from_type(&deps, &config.metadata_type, token_id)?;

    match config.metadata_type {
        MetadataType::OneToOne | MetadataType::Static => {
            if check_attribute_exists(&metadata, &attribute.trait_type) {
                return Err(ContractError::AttributeAlreadyExists {});
            }
            metadata.attributes.push(attribute);
            METADATA.save(deps.storage, metadata_id, &metadata)?;
        }
        MetadataType::Dynamic => {
            if check_attribute_exists(&metadata, &attribute.trait_type) {
                return Err(ContractError::AttributeAlreadyExists {});
            }
            metadata.attributes.push(attribute);
            DYNAMIC_METADATA.save(deps.storage, token_id, &metadata)?;
        }
    };

    Ok(Response::new().add_attribute("action", "execute_add_attribute"))
}

fn execute_update_attribute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: u32,
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
    // check_metadata_lock(&deps, &config, &token_id)?;

    let (metadata_id, mut metadata) =
        get_metadata_from_type(&deps, &config.metadata_type, token_id)?;

    match config.metadata_type {
        MetadataType::OneToOne | MetadataType::Static => {
            if !check_attribute_exists(&metadata, &attribute.trait_type) {
                return Err(ContractError::AttributeNotFound {});
            }

            for item in metadata.attributes.iter_mut() {
                if item.trait_type == attribute.trait_type {
                    *item = attribute;
                    break;
                }
            }
            METADATA.save(deps.storage, metadata_id, &metadata)?;
        }
        MetadataType::Dynamic => {
            if !check_attribute_exists(&metadata, &attribute.trait_type) {
                return Err(ContractError::AttributeNotFound {});
            }

            for item in metadata.attributes.iter_mut() {
                if item.trait_type == attribute.trait_type {
                    *item = attribute;
                    break;
                }
            }
            DYNAMIC_METADATA.save(deps.storage, token_id, &metadata)?;
        }
    };

    Ok(Response::new().add_attribute("action", "execute_update_attribute"))
}

fn execute_remove_attribute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: u32,
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
    // check_metadata_lock(&deps, &config, &token_id)?;

    let (metadata_id, mut metadata) =
        get_metadata_from_type(&deps, &config.metadata_type, token_id)?;

    match config.metadata_type {
        MetadataType::OneToOne | MetadataType::Static => {
            if !check_attribute_exists(&metadata, &trait_type) {
                return Err(ContractError::AttributeNotFound {});
            }

            metadata.attributes = metadata
                .attributes
                .into_iter()
                .filter(|a| a.trait_type != trait_type)
                .collect::<Vec<Trait>>();
            METADATA.save(deps.storage, metadata_id, &metadata)?;
        }
        MetadataType::Dynamic => {
            if !check_attribute_exists(&metadata, &trait_type) {
                return Err(ContractError::AttributeNotFound {});
            }

            metadata.attributes = metadata
                .attributes
                .into_iter()
                .filter(|a| a.trait_type != trait_type)
                .collect::<Vec<Trait>>();
            DYNAMIC_METADATA.save(deps.storage, token_id, &metadata)?;
        }
    };

    Ok(Response::new().add_attribute("action", "execute_remove_attribute"))
}

// fn check_metadata_lock(
//     deps: &DepsMut,
//     config: &Config,
//     token_id: &str,
// ) -> Result<(), ContractError> {
//     let metadata_lock = METADATA_LOCK.may_load(deps.storage, token_id)?;
//     if config.update_lock || (metadata_lock.is_some() && metadata_lock.unwrap()) {
//         return Err(ContractError::UpdateLocked {});
//     }
//     Ok(())
// }

fn get_metadata_from_type(
    deps: &DepsMut,
    metadata_type: &MetadataType,
    token_id: u32,
) -> Result<(u32, Metadata), ContractError> {
    match metadata_type {
        MetadataType::OneToOne | MetadataType::Static => {
            let metadata_id = STATIC_METADATA.may_load(deps.storage, token_id)?;
            if metadata_id.is_none() {
                return Err(ContractError::MissingMetadata {});
            }
            let metadata_id = metadata_id.unwrap();
            let metadata = METADATA.may_load(deps.storage, metadata_id)?;
            if metadata.is_none() {
                return Err(ContractError::MissingMetadata {});
            }
            Ok((metadata_id, metadata.unwrap()))
        }
        MetadataType::Dynamic => {
            let metadata = DYNAMIC_METADATA.may_load(deps.storage, token_id)?;
            if metadata.is_none() {
                return Err(ContractError::MissingMetadata {});
            }
            Ok((token_id, metadata.unwrap()))
        }
    }
}

fn check_attribute_exists(metadata: &Metadata, trait_type: &String) -> bool {
    let exists = metadata
        .attributes
        .iter()
        .any(|a| a.trait_type == *trait_type);
    exists
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::RawMetadata { metadata_id } => to_binary(&query_raw_metadata(deps, metadata_id)?),
        QueryMsg::Metadata { token_id } => to_binary(&query_metadata(deps, token_id)?),
        QueryMsg::RawMetadatas { start_after, limit } => {
            to_binary(&query_raw_metadatas(deps, start_after, limit)?)
        }
        QueryMsg::Metadatas {
            metadata_type,
            start_after,
            limit,
        } => to_binary(&query_metadatas(deps, metadata_type, start_after, limit)?),
        // QueryMsg::MetadataLock { token_id } => to_binary(&query_metadata_lock(deps, token_id)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ResponseWrapper<Config>> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ResponseWrapper::new("config", config))
}

fn query_raw_metadata(deps: Deps, metadata_id: u32) -> StdResult<ResponseWrapper<Metadata>> {
    let metadata = METADATA.load(deps.storage, metadata_id)?;
    Ok(ResponseWrapper::new("raw_metadata", metadata))
}

fn query_metadata(deps: Deps, token_id: u32) -> StdResult<ResponseWrapper<MetadataResponse>> {
    let config = CONFIG.load(deps.storage)?;

    let (metadata_id, metadata) = match config.metadata_type {
        MetadataType::OneToOne | MetadataType::Static => {
            let metadata_id = STATIC_METADATA.load(deps.storage, token_id)?;
            let metadata = METADATA.load(deps.storage, metadata_id)?;
            (metadata_id, metadata)
        }
        MetadataType::Dynamic => {
            let metadata = DYNAMIC_METADATA.load(deps.storage, token_id)?;
            (token_id, metadata)
        }
    };

    Ok(ResponseWrapper::new(
        "metadata",
        MetadataResponse {
            metadata_id,
            metadata,
        },
    ))
}

fn query_raw_metadatas(
    deps: Deps,
    start_after: Option<u32>,
    limit: Option<u8>,
) -> StdResult<ResponseWrapper<Vec<Metadata>>> {
    let limit = limit.unwrap_or(30) as usize;
    let start = start_after.map(Bound::exclusive);
    let metadatas = METADATA
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_, metadata) = item.unwrap();
            metadata
        })
        .collect::<Vec<Metadata>>();
    Ok(ResponseWrapper::new("metadatas", metadatas))
}

fn query_metadatas(
    deps: Deps,
    metadata_type: MetadataType,
    start_after: Option<u32>,
    limit: Option<u8>,
) -> StdResult<ResponseWrapper<Vec<MetadataResponse>>> {
    let limit = limit.unwrap_or(30) as usize;
    let start = start_after.map(Bound::exclusive);

    let metadatas = match metadata_type {
        MetadataType::OneToOne | MetadataType::Static => {
            let data = STATIC_METADATA
                .range(deps.storage, start, None, Order::Ascending)
                .take(limit)
                .map(|item| {
                    let (token_id, metadata_id) = item.unwrap();
                    let metadata = METADATA.load(deps.storage, metadata_id).unwrap();
                    MetadataResponse {
                        metadata_id: token_id,
                        metadata,
                    }
                })
                .collect::<Vec<MetadataResponse>>();
            data
        }
        MetadataType::Dynamic => {
            let data = DYNAMIC_METADATA
                .range(deps.storage, start, None, Order::Ascending)
                .take(limit)
                .map(|item| {
                    let (metadata_id, metadata) = item.unwrap();
                    MetadataResponse {
                        metadata_id,
                        metadata,
                    }
                })
                .collect::<Vec<MetadataResponse>>();
            data
        }
    };

    Ok(ResponseWrapper::new("metadatas", metadatas))
}

// fn query_metadata_lock(deps: Deps, token_id: String) -> StdResult<LockResponse> {
//     let locked = METADATA_LOCK.load(deps.storage, &token_id)?;
//     Ok(LockResponse { locked })
// }
