#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Order, Reply,
    ReplyOn, Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw_storage_plus::Bound;
use cw_utils::parse_reply_instantiate_data;

use komple_types::bundle::Bundles;
use komple_types::module::Modules;
use komple_types::query::ResponseWrapper;
use komple_utils::{check_admin_privileges, query_module_address};
use semver::Version;
use token_contract::msg::{ExecuteMsg as TokenExecuteMsg, InstantiateMsg as TokenInstantiateMsg};

use permission_module::msg::ExecuteMsg as PermissionExecuteMsg;

use crate::error::ContractError;
use crate::msg::{BundlesResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, MintMsg, QueryMsg};
use crate::state::{
    Config, BUNDLE_ADDRS, BUNDLE_ID, BUNDLE_TYPES, CONFIG, COLLECTION_ADDR, LINKED_BUNDLES,
    OPERATORS,
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
        public_bundle_creation: false,
        mint_lock: false,
    };
    CONFIG.save(deps.storage, &config)?;

    BUNDLE_ID.save(deps.storage, &0)?;

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
        ExecuteMsg::CreateBundle {
            code_id,
            token_instantiate_msg,
            linked_bundles,
        } => execute_create_bundle(
            deps,
            env,
            info,
            code_id,
            token_instantiate_msg,
            linked_bundles,
        ),
        ExecuteMsg::UpdatePublicBundleCreation {
            public_bundle_creation,
        } => execute_update_public_bundle_creation(deps, env, info, public_bundle_creation),
        ExecuteMsg::UpdateMintLock { lock } => execute_update_mint_lock(deps, env, info, lock),
        ExecuteMsg::Mint {
            bundle_id,
            metadata_id,
        } => execute_mint(deps, env, info, bundle_id, metadata_id),
        ExecuteMsg::MintTo {
            bundle_id,
            recipient,
            metadata_id,
        } => execute_mint_to(deps, env, info, bundle_id, recipient, metadata_id),
        ExecuteMsg::PermissionMint {
            permission_msg,
            bundle_ids,
            metadata_ids,
        } => execute_permission_mint(deps, env, info, permission_msg, bundle_ids, metadata_ids),
        ExecuteMsg::UpdateOperators { addrs } => execute_update_operators(deps, env, info, addrs),
        ExecuteMsg::UpdateLinkedBundles {
            bundle_id,
            linked_bundles,
        } => execute_update_linked_bundles(deps, env, info, bundle_id, linked_bundles),
    }
}

pub fn execute_create_bundle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    code_id: u64,
    token_instantiate_msg: TokenInstantiateMsg,
    linked_bundles: Option<Vec<u32>>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if !config.public_bundle_creation {
        let collection_addr = COLLECTION_ADDR.may_load(deps.storage)?;
        let operators = OPERATORS.may_load(deps.storage)?;

        check_admin_privileges(
            &info.sender,
            &env.contract.address,
            &config.admin,
            collection_addr,
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

    let bundle_id = (BUNDLE_ID.load(deps.storage)?) + 1;

    if linked_bundles.is_some() {
        check_bundle_ids_exists(&deps, &linked_bundles.clone().unwrap())?;

        LINKED_BUNDLES.save(deps.storage, bundle_id, &linked_bundles.unwrap())?;
    }

    BUNDLE_TYPES.update(
        deps.storage,
        token_instantiate_msg.bundle_info.bundle_type.as_str(),
        |value| -> StdResult<Vec<u32>> {
            match value {
                Some(mut id_list) => {
                    id_list.push(bundle_id);
                    Ok(id_list)
                }
                None => Ok(vec![bundle_id]),
            }
        },
    )?;
    BUNDLE_ID.save(deps.storage, &bundle_id)?;

    Ok(Response::new()
        .add_submessage(sub_msg)
        .add_attribute("action", "create_bundle"))
}

pub fn execute_update_public_bundle_creation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    public_bundle_creation: bool,
) -> Result<Response, ContractError> {
    let collection_addr = COLLECTION_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let mut config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        collection_addr,
        operators,
    )?;

    config.public_bundle_creation = public_bundle_creation;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_public_bundle_creation")
        .add_attribute("public_bundle_creation", public_bundle_creation.to_string()))
}

pub fn execute_update_mint_lock(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    lock: bool,
) -> Result<Response, ContractError> {
    let collection_addr = COLLECTION_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let mut config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        collection_addr,
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
    bundle_id: u32,
    metadata_id: Option<u32>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.mint_lock {
        return Err(ContractError::LockedMint {});
    }

    let mint_msg = vec![MintMsg {
        bundle_id,
        owner: info.sender.to_string(),
        metadata_id,
    }];

    _execute_mint(deps, info.clone(), "execute_mint", mint_msg)
}

fn execute_mint_to(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bundle_id: u32,
    recipient: String,
    metadata_id: Option<u32>,
) -> Result<Response, ContractError> {
    let collection_addr = COLLECTION_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        collection_addr,
        operators,
    )?;

    let owner = deps.api.addr_validate(&recipient)?;

    let mint_msg = vec![MintMsg {
        bundle_id,
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
    bundle_ids: Vec<u32>,
    metadata_ids: Option<Vec<u32>>,
) -> Result<Response, ContractError> {
    let collection_addr = COLLECTION_ADDR.load(deps.storage)?;
    let permission_module_addr =
        query_module_address(&deps.querier, &collection_addr, Modules::PermissionModule)?;

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

    if metadata_ids.is_some() && metadata_ids.as_ref().unwrap().len() != bundle_ids.len() {
        return Err(ContractError::InvalidMetadataIds {});
    }

    for (index, bundle_id) in bundle_ids.iter().enumerate() {
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::MintTo {
                bundle_id: *bundle_id,
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
        let bundle_addr = BUNDLE_ADDRS.load(deps.storage, msg.bundle_id)?;

        let mint_msg = TokenExecuteMsg::Mint {
            owner: msg.owner.clone(),
            metadata_id: msg.metadata_id,
        };
        let msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: bundle_addr.to_string(),
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
    let collection_addr = COLLECTION_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        collection_addr,
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

fn execute_update_linked_bundles(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bundle_id: u32,
    linked_bundles: Vec<u32>,
) -> Result<Response, ContractError> {
    let collection_addr = COLLECTION_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        collection_addr,
        operators,
    )?;

    if linked_bundles.contains(&bundle_id) {
        return Err(ContractError::SelfLinkedBundle {});
    };

    let mut ids_to_check = vec![bundle_id];
    ids_to_check.extend(&linked_bundles);
    check_bundle_ids_exists(&deps, &ids_to_check)?;

    LINKED_BUNDLES.save(deps.storage, bundle_id, &linked_bundles)?;

    Ok(Response::new().add_attribute("action", "execute_update_linked_bundles"))
}

fn check_bundle_ids_exists(deps: &DepsMut, bundle_ids: &Vec<u32>) -> Result<(), ContractError> {
    let existing_ids = BUNDLE_ADDRS
        .keys(deps.storage, None, None, Order::Ascending)
        .map(|id| id.unwrap())
        .collect::<Vec<u32>>();

    for bundle_id in bundle_ids {
        if !existing_ids.contains(bundle_id) {
            return Err(ContractError::InvalidBundleId {});
        }
    }

    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::BundleAddress(bundle_id) => to_binary(&query_bundle_address(deps, bundle_id)?),
        QueryMsg::Operators {} => to_binary(&query_operators(deps)?),
        QueryMsg::BundleTypes(bundle_type) => to_binary(&query_bundle_types(deps, bundle_type)?),
        QueryMsg::LinkedBundles { bundle_id } => to_binary(&query_linked_bundles(deps, bundle_id)?),
        QueryMsg::Bundles { start_after, limit } => {
            to_binary(&query_bundles(deps, start_after, limit)?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<ResponseWrapper<Config>> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ResponseWrapper::new("config", config))
}

fn query_bundle_address(deps: Deps, bundle_id: u32) -> StdResult<ResponseWrapper<String>> {
    let addr = BUNDLE_ADDRS.load(deps.storage, bundle_id)?;
    Ok(ResponseWrapper::new("bundle_address", addr.to_string()))
}

fn query_operators(deps: Deps) -> StdResult<ResponseWrapper<Vec<String>>> {
    let addrs = OPERATORS.may_load(deps.storage)?;
    let addrs = match addrs {
        Some(addrs) => addrs.iter().map(|a| a.to_string()).collect(),
        None => vec![],
    };
    Ok(ResponseWrapper::new("operators", addrs))
}

fn query_bundle_types(deps: Deps, bundle_type: Bundles) -> StdResult<ResponseWrapper<Vec<u32>>> {
    let bundle_ids = BUNDLE_TYPES.may_load(deps.storage, bundle_type.as_str())?;
    let bundle_ids = match bundle_ids {
        Some(ids) => ids,
        None => vec![],
    };
    Ok(ResponseWrapper::new("bundle_types", bundle_ids))
}

fn query_linked_bundles(deps: Deps, bundle_id: u32) -> StdResult<ResponseWrapper<Vec<u32>>> {
    let linked_bundle_ids = LINKED_BUNDLES.may_load(deps.storage, bundle_id)?;
    let linked_bundle_ids = match linked_bundle_ids {
        Some(linked_bundle_ids) => linked_bundle_ids,
        None => vec![],
    };
    Ok(ResponseWrapper::new(
        "linked_bundles",
        linked_bundle_ids,
    ))
}

fn query_bundles(
    deps: Deps,
    start_after: Option<u32>,
    limit: Option<u8>,
) -> StdResult<ResponseWrapper<Vec<BundlesResponse>>> {
    let limit = limit.unwrap_or(30) as usize;
    let start = start_after.map(Bound::exclusive);
    let bundles = BUNDLE_ADDRS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (bundle_id, address) = item.unwrap();
            BundlesResponse {
                bundle_id,
                address: address.to_string(),
            }
        })
        .collect::<Vec<BundlesResponse>>();
    Ok(ResponseWrapper::new("bundles", bundles))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != TOKEN_INSTANTIATE_REPLY_ID {
        return Err(ContractError::InvalidReplyID {});
    }

    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            let bundle_id = BUNDLE_ID.load(deps.storage)?;
            BUNDLE_ADDRS.save(
                deps.storage,
                bundle_id,
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
