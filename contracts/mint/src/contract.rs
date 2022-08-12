#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Order, Reply,
    ReplyOn, Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw_utils::parse_reply_instantiate_data;

use komple_types::collection::Collections;
use komple_types::module::Modules;
use komple_types::query::ResponseWrapper;
use komple_utils::{check_admin_privileges, query_module_address};
use semver::Version;
use token_contract::msg::{ExecuteMsg as TokenExecuteMsg, InstantiateMsg as TokenInstantiateMsg};

use permission_module::msg::ExecuteMsg as PermissionExecuteMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, MintMsg, QueryMsg};
use crate::state::{
    Config, COLLECTION_ADDRS, COLLECTION_ID, COLLECTION_TYPES, CONFIG, CONTROLLER_ADDR,
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

    CONTROLLER_ADDR.save(deps.storage, &info.sender)?;

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
        let controller_addr = CONTROLLER_ADDR.may_load(deps.storage)?;
        let operators = OPERATORS.may_load(deps.storage)?;

        check_admin_privileges(
            &info.sender,
            &env.contract.address,
            &config.admin,
            controller_addr,
            operators,
        )?;
    };

    let mut msg = token_instantiate_msg.clone();
    msg.admin = config.admin.to_string();
    msg.token_info.minter = env.contract.address.to_string();

    // Instantiate token contract
    let sub_msg: SubMsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id,
            msg: to_binary(&msg)?,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: String::from("komple Framework Token Contract"),
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

    COLLECTION_TYPES.update(
        deps.storage,
        token_instantiate_msg
            .collection_info
            .collection_type
            .as_str(),
        |value| -> StdResult<Vec<u32>> {
            match value {
                Some(mut id_list) => {
                    id_list.push(collection_id);
                    Ok(id_list)
                }
                None => Ok(vec![collection_id]),
            }
        },
    )?;
    COLLECTION_ID.save(deps.storage, &collection_id)?;

    Ok(Response::new()
        .add_submessage(sub_msg)
        .add_attribute("action", "create_collection"))
}

pub fn execute_update_public_collection_creation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    public_collection_creation: bool,
) -> Result<Response, ContractError> {
    let controller_addr = CONTROLLER_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let mut config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        controller_addr,
        operators,
    )?;

    config.public_collection_creation = public_collection_creation;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_public_collection_creation")
        .add_attribute(
            "public_collection_creation",
            public_collection_creation.to_string(),
        ))
}

pub fn execute_update_mint_lock(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    lock: bool,
) -> Result<Response, ContractError> {
    let controller_addr = CONTROLLER_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let mut config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        controller_addr,
        operators,
    )?;

    config.mint_lock = lock;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_mint_lock")
        .add_attribute("mint_lock", lock.to_string()))
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
    let controller_addr = CONTROLLER_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        controller_addr,
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
    let controller_addr = CONTROLLER_ADDR.load(deps.storage)?;
    let permission_module_addr =
        query_module_address(&deps.querier, &controller_addr, Modules::PermissionModule)?;

    let mut msgs: Vec<CosmosMsg> = vec![];

    let permission_msg = PermissionExecuteMsg::Check {
        module: Modules::MintModule,
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
    let mut mint_msgs: Vec<CosmosMsg> = vec![];

    for msg in msgs {
        let collection_addr = COLLECTION_ADDRS.load(deps.storage, msg.collection_id)?;

        let mint_msg = TokenExecuteMsg::Mint {
            owner: msg.owner.clone(),
            metadata_id: msg.metadata_id,
        };
        let msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: collection_addr.to_string(),
            msg: to_binary(&mint_msg)?,
            funds: info.funds.clone(),
        });
        mint_msgs.push(msg);
    }

    Ok(Response::new()
        .add_messages(mint_msgs)
        .add_attribute("action", action))
}

fn execute_update_operators(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    addrs: Vec<String>,
) -> Result<Response, ContractError> {
    let controller_addr = CONTROLLER_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        controller_addr,
        operators,
    )?;

    let addrs = addrs
        .iter()
        .map(|addr| -> StdResult<Addr> {
            let addr = deps.api.addr_validate(addr)?;
            Ok(addr)
        })
        .collect::<StdResult<Vec<Addr>>>()?;

    OPERATORS.save(deps.storage, &addrs)?;

    Ok(Response::new().add_attribute("action", "execute_update_operators"))
}

fn execute_update_linked_collections(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_id: u32,
    linked_collections: Vec<u32>,
) -> Result<Response, ContractError> {
    let controller_addr = CONTROLLER_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        controller_addr,
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

fn check_collection_ids_exists(
    deps: &DepsMut,
    collection_ids: &Vec<u32>,
) -> Result<(), ContractError> {
    let existing_ids = COLLECTION_ADDRS
        .keys(deps.storage, None, None, Order::Ascending)
        .map(|id| id.unwrap())
        .collect::<Vec<u32>>();

    for collection_id in collection_ids {
        if !existing_ids.contains(collection_id) {
            return Err(ContractError::InvalidCollectionId {});
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
        QueryMsg::CollectionTypes(collection_type) => {
            to_binary(&query_collection_types(deps, collection_type)?)
        }
        QueryMsg::LinkedCollections { collection_id } => {
            to_binary(&query_linked_collections(deps, collection_id)?)
        }
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

fn query_collection_types(
    deps: Deps,
    collection_type: Collections,
) -> StdResult<ResponseWrapper<Vec<u32>>> {
    let collection_ids = COLLECTION_TYPES.may_load(deps.storage, collection_type.as_str())?;
    let collection_ids = match collection_ids {
        Some(ids) => ids,
        None => vec![],
    };
    Ok(ResponseWrapper::new("collection_types", collection_ids))
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
