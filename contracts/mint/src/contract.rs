#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Attribute, Binary, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo, Order,
    Reply, ReplyOn, Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw_storage_plus::Bound;
use cw_utils::parse_reply_instantiate_data;

use komple_token_module::{helper::KompleTokenModule, msg::InstantiateMsg as TokenInstantiateMsg};
use komple_types::module::Modules;
use komple_types::query::ResponseWrapper;
use komple_utils::event::EventHelper;
use komple_utils::{check_admin_privileges, storage::StorageHelper};
use semver::Version;

use komple_permission_module::msg::ExecuteMsg as PermissionExecuteMsg;

use crate::error::ContractError;
use crate::msg::{CollectionsResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, MintMsg, QueryMsg};
use crate::state::{
    Config, BLACKLIST_COLLECTION_ADDRS, COLLECTION_ADDRS, COLLECTION_ID, CONFIG, HUB_ADDR,
    LINKED_COLLECTIONS, OPERATORS,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:komple-mint-module";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const TOKEN_INSTANTIATE_REPLY_ID: u64 = 1;

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
        public_collection_creation: false,
        mint_lock: false,
    };
    CONFIG.save(deps.storage, &config)?;

    COLLECTION_ID.save(deps.storage, &0)?;

    HUB_ADDR.save(deps.storage, &info.sender)?;

    Ok(Response::new().add_event(
        EventHelper::new("komple_mint_module")
            .add_attribute("action", "instantiate")
            .add_attribute("admin", config.admin.to_string())
            .add_attribute(
                "public_collection_creation",
                config.public_collection_creation.to_string(),
            )
            .add_attribute("mint_lock", config.mint_lock.to_string())
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
    match msg {
        ExecuteMsg::CreateCollection {
            code_id,
            token_instantiate_msg,
            linked_collections,
        } => execute_create_collection(
            deps,
            env,
            info,
            code_id,
            token_instantiate_msg,
            linked_collections,
        ),
        ExecuteMsg::UpdatePublicCollectionCreation {
            public_collection_creation,
        } => execute_update_public_collection_creation(deps, env, info, public_collection_creation),
        ExecuteMsg::UpdateMintLock { lock } => execute_update_mint_lock(deps, env, info, lock),
        ExecuteMsg::Mint {
            collection_id,
            metadata_id,
        } => execute_mint(deps, env, info, collection_id, metadata_id),
        ExecuteMsg::MintTo {
            collection_id,
            recipient,
            metadata_id,
        } => execute_mint_to(deps, env, info, collection_id, recipient, metadata_id),
        ExecuteMsg::PermissionMint {
            permission_msg,
            collection_ids,
            metadata_ids,
        } => execute_permission_mint(
            deps,
            env,
            info,
            permission_msg,
            collection_ids,
            metadata_ids,
        ),
        ExecuteMsg::UpdateOperators { addrs } => execute_update_operators(deps, env, info, addrs),
        ExecuteMsg::UpdateLinkedCollections {
            collection_id,
            linked_collections,
        } => execute_update_linked_collections(deps, env, info, collection_id, linked_collections),
        ExecuteMsg::WhitelistCollection { collection_id } => {
            execute_whitelist_collection(deps, env, info, collection_id)
        }
        ExecuteMsg::BlacklistCollection { collection_id } => {
            execute_blacklist_collection(deps, env, info, collection_id)
        }
    }
}

pub fn execute_create_collection(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    code_id: u64,
    token_instantiate_msg: TokenInstantiateMsg,
    linked_collections: Option<Vec<u32>>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if !config.public_collection_creation {
        let hub_addr = HUB_ADDR.may_load(deps.storage)?;
        let operators = OPERATORS.may_load(deps.storage)?;

        check_admin_privileges(
            &info.sender,
            &env.contract.address,
            &config.admin,
            hub_addr,
            operators,
        )?;
    };

    let mut msg = token_instantiate_msg.clone();
    msg.admin = config.admin.to_string();
    msg.creator = info.sender.to_string();
    msg.token_info.minter = env.contract.address.to_string();

    // Instantiate token contract
    let sub_msg: SubMsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id,
            msg: to_binary(&msg)?,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: String::from("Komple Framework Token Module"),
        }
        .into(),
        id: TOKEN_INSTANTIATE_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    let collection_id = (COLLECTION_ID.load(deps.storage)?) + 1;

    if linked_collections.is_some() {
        check_collection_ids_exists(&deps, &linked_collections.clone().unwrap())?;

        LINKED_COLLECTIONS.save(deps.storage, collection_id, &linked_collections.unwrap())?;
    }

    COLLECTION_ID.save(deps.storage, &collection_id)?;

    Ok(Response::new().add_submessage(sub_msg).add_event(
        EventHelper::new("komple_mint_module")
            .add_attribute("action", "create_collection")
            .get(),
    ))
}

pub fn execute_update_public_collection_creation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    public_collection_creation: bool,
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

    config.public_collection_creation = public_collection_creation;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_event(
        EventHelper::new("komple_mint_module")
            .add_attribute("action", "update_public_collection_creation")
            .add_attribute(
                "public_collection_creation",
                public_collection_creation.to_string(),
            )
            .get(),
    ))
}

pub fn execute_update_mint_lock(
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

    config.mint_lock = lock;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_event(
        EventHelper::new("komple_mint_module")
            .add_attribute("action", "update_mint_lock")
            .add_attribute("mint_lock", lock.to_string())
            .get(),
    ))
}

fn execute_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_id: u32,
    metadata_id: Option<u32>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.mint_lock {
        return Err(ContractError::LockedMint {});
    }

    let mint_msg = vec![MintMsg {
        collection_id,
        owner: info.sender.to_string(),
        metadata_id,
    }];

    _execute_mint(deps, info.clone(), "execute_mint", mint_msg)
}

fn execute_mint_to(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_id: u32,
    recipient: String,
    metadata_id: Option<u32>,
) -> Result<Response, ContractError> {
    let hub_addr = HUB_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        hub_addr,
        operators,
    )?;

    let owner = deps.api.addr_validate(&recipient)?;

    let mint_msg = vec![MintMsg {
        collection_id,
        owner: owner.to_string(),
        metadata_id,
    }];

    _execute_mint(deps, info, "execute_mint_to", mint_msg)
}

fn execute_permission_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    permission_msg: Binary,
    collection_ids: Vec<u32>,
    metadata_ids: Option<Vec<u32>>,
) -> Result<Response, ContractError> {
    let hub_addr = HUB_ADDR.load(deps.storage)?;
    let permission_module_addr =
        StorageHelper::query_module_address(&deps.querier, &hub_addr, Modules::Permission)?;

    let mut msgs: Vec<CosmosMsg> = vec![];

    let permission_msg = PermissionExecuteMsg::Check {
        module: Modules::Mint.to_string(),
        msg: permission_msg,
    };
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: permission_module_addr.to_string(),
        msg: to_binary(&permission_msg)?,
        funds: info.funds.clone(),
    }));

    if metadata_ids.is_some() && metadata_ids.as_ref().unwrap().len() != collection_ids.len() {
        return Err(ContractError::InvalidMetadataIds {});
    }

    for (index, collection_id) in collection_ids.iter().enumerate() {
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::MintTo {
                collection_id: *collection_id,
                recipient: info.sender.to_string(),
                metadata_id: metadata_ids.as_ref().and_then(|ids| Some(ids[index])),
            })?,
            funds: info.funds.clone(),
        }))
    }

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "execute_permission_mint"))
}

fn _execute_mint(
    deps: DepsMut,
    info: MessageInfo,
    action: &str,
    msgs: Vec<MintMsg>,
) -> Result<Response, ContractError> {
    let mut mint_msgs: Vec<WasmMsg> = vec![];

    let mut event_attributes: Vec<Attribute> = vec![];

    for (index, msg) in msgs.iter().enumerate() {
        event_attributes.push(Attribute {
            key: format!("mint_msg_{}", index.to_string()),
            value: format!("collection_id/{}", msg.collection_id),
        });
        if msg.metadata_id.is_some() {
            event_attributes.push(Attribute {
                key: format!("mint_msg_{}", index.to_string()),
                value: format!(
                    "metadata_id/{}",
                    msg.metadata_id.as_ref().unwrap().to_string()
                ),
            });
        }
        event_attributes.push(Attribute {
            key: format!("mint_msg_{}", index.to_string()),
            value: format!("owner/{}", msg.owner),
        });

        let collection_addr = COLLECTION_ADDRS.load(deps.storage, msg.collection_id)?;

        let msg = KompleTokenModule(collection_addr).mint_msg(
            msg.owner.clone(),
            msg.metadata_id,
            info.funds.clone(),
        )?;
        mint_msgs.push(msg);
    }

    Ok(Response::new().add_messages(mint_msgs).add_event(
        EventHelper::new("komple_mint_module")
            .add_attribute("action", action)
            .add_attributes(event_attributes)
            .get(),
    ))
}

fn execute_update_operators(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut addrs: Vec<String>,
) -> Result<Response, ContractError> {
    let hub_addr = HUB_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        hub_addr,
        operators,
    )?;

    addrs.sort_unstable();
    addrs.dedup();

    let mut event_attributes: Vec<Attribute> = vec![];

    let addrs = addrs
        .iter()
        .map(|addr| -> StdResult<Addr> {
            let addr = deps.api.addr_validate(addr)?;
            event_attributes.push(Attribute {
                key: "addrs".to_string(),
                value: addr.to_string(),
            });
            Ok(addr)
        })
        .collect::<StdResult<Vec<Addr>>>()?;

    OPERATORS.save(deps.storage, &addrs)?;

    Ok(Response::new().add_event(
        Event::new("komple_mint_module")
            .add_attribute("action".to_string(), "update_operators".to_string())
            .add_attributes(event_attributes),
    ))
}

fn execute_update_linked_collections(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_id: u32,
    linked_collections: Vec<u32>,
) -> Result<Response, ContractError> {
    let hub_addr = HUB_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        hub_addr,
        operators,
    )?;

    if linked_collections.contains(&collection_id) {
        return Err(ContractError::SelfLinkedCollection {});
    };

    let mut ids_to_check = vec![collection_id];
    ids_to_check.extend(&linked_collections);
    check_collection_ids_exists(&deps, &ids_to_check)?;

    LINKED_COLLECTIONS.save(deps.storage, collection_id, &linked_collections)?;

    Ok(Response::new().add_attribute("action", "execute_update_linked_collections"))
}

fn execute_whitelist_collection(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_id: u32,
) -> Result<Response, ContractError> {
    let hub_addr = HUB_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        hub_addr,
        operators,
    )?;

    if COLLECTION_ADDRS.has(deps.storage, collection_id) {
        return Err(ContractError::AlreadyWhitelistlisted {});
    };

    let collection_addr = BLACKLIST_COLLECTION_ADDRS.may_load(deps.storage, collection_id)?;
    if collection_addr.is_none() {
        return Err(ContractError::CollectionIdNotFound {});
    };

    BLACKLIST_COLLECTION_ADDRS.remove(deps.storage, collection_id);
    COLLECTION_ADDRS.save(deps.storage, collection_id, &collection_addr.unwrap())?;

    Ok(Response::new().add_event(
        EventHelper::new("komple_mint_module")
            .add_attribute("action", "whitelist_collection")
            .add_attribute("collection_id", collection_id.to_string())
            .get(),
    ))
}

fn execute_blacklist_collection(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_id: u32,
) -> Result<Response, ContractError> {
    let hub_addr = HUB_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        hub_addr,
        operators,
    )?;

    if BLACKLIST_COLLECTION_ADDRS.has(deps.storage, collection_id) {
        return Err(ContractError::AlreadyBlacklisted {});
    };

    let collection_addr = COLLECTION_ADDRS.may_load(deps.storage, collection_id)?;
    if collection_addr.is_none() {
        return Err(ContractError::CollectionIdNotFound {});
    };

    COLLECTION_ADDRS.remove(deps.storage, collection_id);
    BLACKLIST_COLLECTION_ADDRS.save(deps.storage, collection_id, &collection_addr.unwrap())?;

    Ok(Response::new().add_event(
        EventHelper::new("komple_mint_module")
            .add_attribute("action", "blacklist_collection")
            .add_attribute("collection_id", collection_id.to_string())
            .get(),
    ))
}

fn check_collection_ids_exists(
    deps: &DepsMut,
    collection_ids: &Vec<u32>,
) -> Result<(), ContractError> {
    for collection_id in collection_ids {
        if !COLLECTION_ADDRS.has(deps.storage, *collection_id) {
            return Err(ContractError::CollectionIdNotFound {});
        }
    }
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::CollectionAddress(collection_id) => {
            to_binary(&query_collection_address(deps, collection_id)?)
        }
        QueryMsg::Operators {} => to_binary(&query_operators(deps)?),
        QueryMsg::LinkedCollections { collection_id } => {
            to_binary(&query_linked_collections(deps, collection_id)?)
        }
        QueryMsg::Collections {
            blacklist,
            start_after,
            limit,
        } => to_binary(&query_collections(deps, blacklist, start_after, limit)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ResponseWrapper<Config>> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ResponseWrapper::new("config", config))
}

fn query_collection_address(deps: Deps, collection_id: u32) -> StdResult<ResponseWrapper<String>> {
    let addr = COLLECTION_ADDRS.load(deps.storage, collection_id)?;
    Ok(ResponseWrapper::new("collection_address", addr.to_string()))
}

fn query_operators(deps: Deps) -> StdResult<ResponseWrapper<Vec<String>>> {
    let addrs = OPERATORS.may_load(deps.storage)?;
    let addrs = match addrs {
        Some(addrs) => addrs.iter().map(|a| a.to_string()).collect(),
        None => vec![],
    };
    Ok(ResponseWrapper::new("operators", addrs))
}

fn query_linked_collections(
    deps: Deps,
    collection_id: u32,
) -> StdResult<ResponseWrapper<Vec<u32>>> {
    let linked_collection_ids = LINKED_COLLECTIONS.may_load(deps.storage, collection_id)?;
    let linked_collection_ids = match linked_collection_ids {
        Some(linked_collection_ids) => linked_collection_ids,
        None => vec![],
    };
    Ok(ResponseWrapper::new(
        "linked_collections",
        linked_collection_ids,
    ))
}

fn query_collections(
    deps: Deps,
    blacklist: bool,
    start_after: Option<u32>,
    limit: Option<u8>,
) -> StdResult<ResponseWrapper<Vec<CollectionsResponse>>> {
    let limit = limit.unwrap_or(30) as usize;
    let start = start_after.map(Bound::exclusive);

    let collections_state = match blacklist {
        true => BLACKLIST_COLLECTION_ADDRS,
        false => COLLECTION_ADDRS,
    };

    let collections = collections_state
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (collection_id, address) = item.unwrap();
            CollectionsResponse {
                collection_id,
                address: address.to_string(),
            }
        })
        .collect::<Vec<CollectionsResponse>>();

    Ok(ResponseWrapper::new("collections", collections))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != TOKEN_INSTANTIATE_REPLY_ID {
        return Err(ContractError::InvalidReplyID {});
    }

    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            let collection_id = COLLECTION_ID.load(deps.storage)?;
            COLLECTION_ADDRS.save(
                deps.storage,
                collection_id,
                &Addr::unchecked(res.contract_address),
            )?;
            Ok(Response::default().add_attribute("action", "instantiate_token_reply"))
        }
        Err(_) => Err(ContractError::TokenInstantiateError {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version: Version = CONTRACT_VERSION.parse()?;
    let contract_version: cw2::ContractVersion = get_contract_version(deps.storage)?;
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
