#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Attribute, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdError, StdResult,
};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use cw_storage_plus::Bound;
use komple_types::modules::metadata::Metadata as MetadataType;
use komple_types::shared::events::MetadataEventAttributes;
use komple_types::shared::query::ResponseWrapper;
use komple_types::shared::RegisterMsg;
use komple_utils::check_admin_privileges;
use komple_utils::response::{EventHelper, ResponseHelper};
use komple_utils::shared::execute_update_operators;
use semver::Version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MetadataResponse, MigrateMsg, QueryMsg};
use crate::state::{
    Config, MetaInfo, Metadata, Trait, COLLECTION_ADDR, CONFIG, DYNAMIC_LINKED_METADATA,
    LINKED_METADATA, METADATA, METADATA_ID, OPERATORS,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:komple-metadata-module";
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
        metadata_type: data.metadata_type,
    };

    CONFIG.save(deps.storage, &config)?;

    COLLECTION_ADDR.save(deps.storage, &info.sender)?;

    METADATA_ID.save(deps.storage, &0)?;

    Ok(
        ResponseHelper::new_module("metadata", "instantiate").add_event(
            EventHelper::new("metadata_instantiate")
                .add_attribute("admin", config.admin)
                .add_attribute("metadata_type", config.metadata_type.to_string())
                .add_attribute("collection_addr", info.sender)
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
        ExecuteMsg::AddMetadata {
            meta_info,
            attributes,
        } => execute_add_metadata(deps, env, info, meta_info, attributes),
        ExecuteMsg::LinkMetadata {
            token_id,
            metadata_id,
        } => execute_link_metadata(deps, env, info, token_id, metadata_id),
        ExecuteMsg::UnlinkMetadata { token_id } => {
            execute_unlink_metadata(deps, env, info, token_id)
        }
        ExecuteMsg::UpdateMetaInfo {
            raw_metadata,
            id,
            meta_info,
        } => execute_update_meta_info(deps, env, info, raw_metadata, id, meta_info),
        ExecuteMsg::AddAttribute {
            raw_metadata,
            id,
            attribute,
        } => execute_add_attribute(deps, env, info, raw_metadata, id, attribute),
        ExecuteMsg::UpdateAttribute {
            raw_metadata,
            id,
            attribute,
        } => execute_update_attribute(deps, env, info, raw_metadata, id, attribute),
        ExecuteMsg::RemoveAttribute {
            raw_metadata,
            id,
            trait_type,
        } => execute_remove_attribute(deps, env, info, raw_metadata, id, trait_type),
        ExecuteMsg::UpdateOperators { addrs } => {
            let config = CONFIG.load(deps.storage)?;
            let res = execute_update_operators(
                deps,
                info,
                "metadata",
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

    // Create the metadata object
    let metadata = Metadata {
        meta_info,
        attributes,
    };

    // Metadata id is the raw metadata id
    let metadata_id = (METADATA_ID.load(deps.storage)?) + 1;

    METADATA.save(deps.storage, metadata_id, &metadata)?;
    METADATA_ID.save(deps.storage, &metadata_id)?;

    let mut event_attributes: Vec<Attribute> = vec![];

    if !metadata.attributes.is_empty() {
        for attribute in metadata.attributes {
            event_attributes.push(MetadataEventAttributes::new_attribute_attribute(
                attribute.trait_type,
                attribute.value,
            ));
        }
    }

    Ok(
        ResponseHelper::new_module("metadata", "add_metadata").add_event(
            EventHelper::new("metadata_add_metadata")
                .check_add_attribute(
                    &metadata.meta_info.image,
                    "meta_info",
                    MetadataEventAttributes::new_meta_info_value(
                        "image",
                        &metadata.meta_info.image,
                    ),
                )
                .check_add_attribute(
                    &metadata.meta_info.external_url,
                    "meta_info",
                    MetadataEventAttributes::new_meta_info_value(
                        "external_url",
                        &metadata.meta_info.external_url,
                    ),
                )
                .check_add_attribute(
                    &metadata.meta_info.description,
                    "meta_info",
                    MetadataEventAttributes::new_meta_info_value(
                        "description",
                        &metadata.meta_info.description,
                    ),
                )
                .check_add_attribute(
                    &metadata.meta_info.animation_url,
                    "meta_info",
                    MetadataEventAttributes::new_meta_info_value(
                        "animation_url",
                        &metadata.meta_info.animation_url,
                    ),
                )
                .check_add_attribute(
                    &metadata.meta_info.youtube_url,
                    "meta_info",
                    MetadataEventAttributes::new_meta_info_value(
                        "youtube_url",
                        &metadata.meta_info.youtube_url,
                    ),
                )
                .add_attributes(event_attributes)
                .get(),
        ),
    )
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

    // If the metadata type is standard use token id as metadata id
    // Else use the metadata id provided
    let metadata_id = match config.metadata_type {
        MetadataType::Standard => token_id,
        MetadataType::Shared | MetadataType::Dynamic => {
            if metadata_id.is_none() {
                return Err(ContractError::MissingMetadata {});
            }
            metadata_id.unwrap()
        }
    };

    // Get the raw metadata from storage
    let metadata = METADATA.may_load(deps.storage, metadata_id)?;
    if metadata.is_none() {
        return Err(ContractError::MissingMetadata {});
    }

    // Based on the metadata type, add metadata information to correct storage
    match config.metadata_type {
        MetadataType::Standard | MetadataType::Shared => {
            LINKED_METADATA.save(deps.storage, token_id, &metadata_id)?;
        }
        MetadataType::Dynamic => {
            let metadata = metadata.unwrap();
            DYNAMIC_LINKED_METADATA.save(deps.storage, token_id, &metadata)?;
        }
    };

    Ok(
        ResponseHelper::new_module("metadata", "link_metadata").add_event(
            EventHelper::new("metadata_link_metadata")
                .add_attribute("token_id", token_id.to_string())
                .add_attribute("metadata_id", metadata_id.to_string())
                .get(),
        ),
    )
}

fn execute_update_meta_info(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    raw_metadata: bool,
    id: u32,
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

    // If raw metadata is true use the raw metadata storage
    // Else use the linked metadata storage
    let (metadata_id, mut metadata) = match raw_metadata {
        true => {
            let metadata = METADATA.may_load(deps.storage, id)?;
            if metadata.is_none() {
                return Err(ContractError::MissingMetadata {});
            }
            (id, metadata.unwrap())
        }
        false => get_metadata_with_token_id(&deps, &config.metadata_type, id)?,
    };

    metadata.meta_info = meta_info;

    if raw_metadata
        || config.metadata_type == MetadataType::Standard
        || config.metadata_type == MetadataType::Shared
    {
        METADATA.save(deps.storage, metadata_id, &metadata)?;
    } else {
        DYNAMIC_LINKED_METADATA.save(deps.storage, id, &metadata)?;
    }

    Ok(
        ResponseHelper::new_module("metadata", "update_meta_info").add_event(
            EventHelper::new("metadata_update_meta_info")
                .add_attribute("raw_metadata", raw_metadata.to_string())
                .add_attribute("id", id.to_string())
                .check_add_attribute(
                    &metadata.meta_info.image,
                    "meta_info",
                    MetadataEventAttributes::new_meta_info_value(
                        "image",
                        &metadata.meta_info.image,
                    ),
                )
                .check_add_attribute(
                    &metadata.meta_info.external_url,
                    "meta_info",
                    MetadataEventAttributes::new_meta_info_value(
                        "external_url",
                        &metadata.meta_info.external_url,
                    ),
                )
                .check_add_attribute(
                    &metadata.meta_info.description,
                    "meta_info",
                    MetadataEventAttributes::new_meta_info_value(
                        "description",
                        &metadata.meta_info.description,
                    ),
                )
                .check_add_attribute(
                    &metadata.meta_info.animation_url,
                    "meta_info",
                    MetadataEventAttributes::new_meta_info_value(
                        "animation_url",
                        &metadata.meta_info.animation_url,
                    ),
                )
                .check_add_attribute(
                    &metadata.meta_info.youtube_url,
                    "meta_info",
                    MetadataEventAttributes::new_meta_info_value(
                        "youtube_url",
                        &metadata.meta_info.youtube_url,
                    ),
                )
                .get(),
        ),
    )
}

fn execute_add_attribute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    raw_metadata: bool,
    id: u32,
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

    let (metadata_id, mut metadata) = match raw_metadata {
        true => {
            let metadata = METADATA.may_load(deps.storage, id)?;
            if metadata.is_none() {
                return Err(ContractError::MissingMetadata {});
            }
            (id, metadata.unwrap())
        }
        false => get_metadata_with_token_id(&deps, &config.metadata_type, id)?,
    };

    if check_attribute_exists(&metadata, &attribute.trait_type) {
        return Err(ContractError::AttributeAlreadyExists {});
    }
    metadata.attributes.push(attribute.clone());

    if raw_metadata
        || config.metadata_type == MetadataType::Standard
        || config.metadata_type == MetadataType::Shared
    {
        METADATA.save(deps.storage, metadata_id, &metadata)?;
    } else {
        DYNAMIC_LINKED_METADATA.save(deps.storage, id, &metadata)?;
    }

    Ok(
        ResponseHelper::new_module("metadata", "add_attribute").add_event(
            EventHelper::new("metadata_add_attribute")
                .add_attribute("raw_metadata", raw_metadata.to_string())
                .add_attribute("id", id.to_string())
                .add_attributes(vec![MetadataEventAttributes::new_attribute_attribute(
                    attribute.trait_type,
                    attribute.value,
                )])
                .get(),
        ),
    )
}

fn execute_update_attribute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    raw_metadata: bool,
    id: u32,
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

    let (metadata_id, mut metadata) = match raw_metadata {
        true => {
            let metadata = METADATA.may_load(deps.storage, id)?;
            if metadata.is_none() {
                return Err(ContractError::MissingMetadata {});
            }
            (id, metadata.unwrap())
        }
        false => get_metadata_with_token_id(&deps, &config.metadata_type, id)?,
    };

    if !check_attribute_exists(&metadata, &attribute.trait_type) {
        return Err(ContractError::AttributeNotFound {});
    }
    for item in metadata.attributes.iter_mut() {
        if item.trait_type == attribute.trait_type {
            *item = attribute.clone();
            break;
        }
    }

    if raw_metadata
        || config.metadata_type == MetadataType::Standard
        || config.metadata_type == MetadataType::Shared
    {
        METADATA.save(deps.storage, metadata_id, &metadata)?;
    } else {
        DYNAMIC_LINKED_METADATA.save(deps.storage, id, &metadata)?;
    }

    Ok(
        ResponseHelper::new_module("metadata", "update_attribute").add_event(
            EventHelper::new("metadata_update_attribute")
                .add_attribute("raw_metadata", raw_metadata.to_string())
                .add_attribute("id", id.to_string())
                .add_attributes(vec![MetadataEventAttributes::new_attribute_attribute(
                    attribute.trait_type,
                    attribute.value,
                )])
                .get(),
        ),
    )
}

fn execute_remove_attribute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    raw_metadata: bool,
    id: u32,
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

    let (metadata_id, mut metadata) = match raw_metadata {
        true => {
            let metadata = METADATA.may_load(deps.storage, id)?;
            if metadata.is_none() {
                return Err(ContractError::MissingMetadata {});
            }
            (id, metadata.unwrap())
        }
        false => get_metadata_with_token_id(&deps, &config.metadata_type, id)?,
    };

    if !check_attribute_exists(&metadata, &trait_type) {
        return Err(ContractError::AttributeNotFound {});
    }
    metadata.attributes = metadata
        .attributes
        .into_iter()
        .filter(|a| a.trait_type != trait_type)
        .collect::<Vec<Trait>>();

    if raw_metadata
        || config.metadata_type == MetadataType::Standard
        || config.metadata_type == MetadataType::Shared
    {
        METADATA.save(deps.storage, metadata_id, &metadata)?;
    } else {
        DYNAMIC_LINKED_METADATA.save(deps.storage, id, &metadata)?;
    }

    Ok(
        ResponseHelper::new_module("metadata", "remove_attribute").add_event(
            EventHelper::new("metadata_remove_attribute")
                .add_attribute("raw_metadata", raw_metadata.to_string())
                .add_attribute("id", id.to_string())
                .add_attribute("trait_type", trait_type)
                .get(),
        ),
    )
}

fn execute_unlink_metadata(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: u32,
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

    match config.metadata_type {
        MetadataType::Standard | MetadataType::Shared => {
            if !LINKED_METADATA.has(deps.storage, token_id) {
                return Err(ContractError::MissingMetadata {});
            }
            LINKED_METADATA.remove(deps.storage, token_id);
        }
        MetadataType::Dynamic => {
            if !DYNAMIC_LINKED_METADATA.has(deps.storage, token_id) {
                return Err(ContractError::MissingMetadata {});
            }
            DYNAMIC_LINKED_METADATA.remove(deps.storage, token_id);
        }
    }

    Ok(
        ResponseHelper::new_module("metadata", "unlink_metadata").add_event(
            EventHelper::new("metadata_unlink_metadata")
                .add_attribute("token_id", token_id.to_string())
                .get(),
        ),
    )
}

fn get_metadata_with_token_id(
    deps: &DepsMut,
    metadata_type: &MetadataType,
    token_id: u32,
) -> Result<(u32, Metadata), ContractError> {
    match metadata_type {
        MetadataType::Standard | MetadataType::Shared => {
            let metadata_id = LINKED_METADATA.may_load(deps.storage, token_id)?;
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
            let metadata = DYNAMIC_LINKED_METADATA.may_load(deps.storage, token_id)?;
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
        QueryMsg::Metadatas { start_after, limit } => {
            to_binary(&query_metadatas(deps, start_after, limit)?)
        }
        QueryMsg::Operators {} => to_binary(&query_operators(deps)?),
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
        MetadataType::Standard | MetadataType::Shared => {
            let metadata_id = LINKED_METADATA.load(deps.storage, token_id)?;
            let metadata = METADATA.load(deps.storage, metadata_id)?;
            (metadata_id, metadata)
        }
        MetadataType::Dynamic => {
            let metadata = DYNAMIC_LINKED_METADATA.load(deps.storage, token_id)?;
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
) -> StdResult<ResponseWrapper<Vec<MetadataResponse>>> {
    let limit = limit.unwrap_or(30) as usize;
    let start = start_after.map(Bound::exclusive);

    let metadatas = METADATA
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

    Ok(ResponseWrapper::new("metadatas", metadatas))
}

fn query_metadatas(
    deps: Deps,
    start_after: Option<u32>,
    limit: Option<u8>,
) -> StdResult<ResponseWrapper<Vec<MetadataResponse>>> {
    let config = CONFIG.load(deps.storage)?;
    let limit = limit.unwrap_or(30) as usize;
    let start = start_after.map(Bound::exclusive);

    let metadatas = match config.metadata_type {
        MetadataType::Standard | MetadataType::Shared => {
            let data = LINKED_METADATA
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
            let data = DYNAMIC_LINKED_METADATA
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
