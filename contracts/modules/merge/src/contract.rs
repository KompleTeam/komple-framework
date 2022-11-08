#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Attribute, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use komple_mint_module::helper::KompleMintModule;
use komple_permission_module::msg::ExecuteMsg as PermissionExecuteMsg;
use komple_token_module::helper::KompleTokenModule;
use komple_types::events::MergeEventAttributes;
use komple_types::module::Modules;
use komple_types::query::ResponseWrapper;
use komple_types::shared::RegisterMsg;
use komple_utils::response::{EventHelper, ResponseHelper};
use komple_utils::shared::{execute_lock_execute, execute_update_operators};
use komple_utils::{check_admin_privileges, storage::StorageHelper};
use semver::Version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, MergeMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG, EXECUTE_LOCK, HUB_ADDR, OPERATORS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:komple-merge-module";
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

    let config = Config {
        admin,
        merge_lock: false,
    };
    CONFIG.save(deps.storage, &config)?;

    HUB_ADDR.save(deps.storage, &info.sender)?;

    EXECUTE_LOCK.save(deps.storage, &false)?;

    Ok(
        ResponseHelper::new_module("merge", "instantiate").add_event(
            EventHelper::new("merge_instantiate")
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
        ExecuteMsg::UpdateMergeLock { lock } => execute_update_merge_lock(deps, env, info, lock),
        ExecuteMsg::Merge { msg } => execute_merge(deps, env, info, msg),
        ExecuteMsg::PermissionMerge {
            permission_msg,
            merge_msg,
        } => execute_permission_merge(deps, env, info, permission_msg, merge_msg),
        ExecuteMsg::UpdateOperators { addrs } => {
            let config = CONFIG.load(deps.storage)?;
            let res = execute_update_operators(
                deps,
                info,
                "merge",
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
                execute_lock_execute(deps, info, "merge", &env.contract.address, EXECUTE_LOCK);
            match res {
                Ok(res) => Ok(res),
                Err(err) => Err(err.into()),
            }
        }
    }
}

fn execute_update_merge_lock(
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

    config.merge_lock = lock;

    CONFIG.save(deps.storage, &config)?;

    Ok(
        ResponseHelper::new_module("merge", "update_merge_lock").add_event(
            EventHelper::new("merge_update_merge_lock")
                .add_attribute("lock", lock.to_string())
                .get(),
        ),
    )
}

fn execute_merge(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MergeMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.merge_lock {
        return Err(ContractError::MergeLocked {});
    };

    let mut msgs: Vec<WasmMsg> = vec![];

    let mut event_attributes: Vec<Attribute> = vec![];

    make_merge_msg(&deps, &info, &mut event_attributes, msg, &mut msgs)?;

    Ok(ResponseHelper::new_module("merge", "merge")
        .add_messages(msgs)
        .add_event(
            EventHelper::new("merge_merge")
                .add_attributes(event_attributes)
                .get(),
        ))
}

fn execute_permission_merge(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    permission_msg: Binary,
    merge_msg: MergeMsg,
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

    let hub_addr = HUB_ADDR.load(deps.storage)?;
    let permission_module_addr = StorageHelper::query_module_address(
        &deps.querier,
        &hub_addr,
        Modules::Permission.to_string(),
    )?;

    let mut msgs: Vec<WasmMsg> = vec![];

    let permission_msg = PermissionExecuteMsg::Check {
        module: Modules::Merge.to_string(),
        msg: permission_msg,
    };
    msgs.push(WasmMsg::Execute {
        contract_addr: permission_module_addr.to_string(),
        msg: to_binary(&permission_msg)?,
        funds: info.funds.clone(),
    });

    let mut event_attributes: Vec<Attribute> = vec![];

    make_merge_msg(&deps, &info, &mut event_attributes, merge_msg, &mut msgs)?;

    Ok(ResponseHelper::new_module("merge", "permission_merge")
        .add_messages(msgs)
        .add_event(
            EventHelper::new("merge_permission_merge")
                .add_attributes(event_attributes)
                .get(),
        ))
}

/// Constructs the mint and burn messages
fn make_merge_msg(
    deps: &DepsMut,
    info: &MessageInfo,
    event_attributes: &mut Vec<Attribute>,
    merge_msg: MergeMsg,
    msgs: &mut Vec<WasmMsg>,
) -> Result<(), ContractError> {
    let hub_addr = HUB_ADDR.load(deps.storage)?;
    let mint_module_addr =
        StorageHelper::query_module_address(&deps.querier, &hub_addr, Modules::Mint.to_string())?;

    // Throw an error if there are no burn messages
    if merge_msg.burn_ids.is_empty() {
        return Err(ContractError::BurnNotFound {});
    }

    // Pushes the burn_ids inside msgs list
    make_burn_messages(deps, event_attributes, &mint_module_addr, &merge_msg, msgs)?;

    let msg = KompleMintModule(mint_module_addr).admin_mint_msg(
        merge_msg.recipient,
        merge_msg.mint_id,
        merge_msg.metadata_id,
        info.funds.clone(),
    )?;
    msgs.push(msg);

    event_attributes.push(Attribute::new("mint_id", merge_msg.mint_id.to_string()));
    if merge_msg.metadata_id.is_some() {
        event_attributes.push(Attribute::new(
            "metadata_id",
            merge_msg.metadata_id.as_ref().unwrap().to_string(),
        ));
    }

    Ok(())
}

// Loops through the tokens to burn inside merge message and
// creates the messages for burning the tokens
fn make_burn_messages(
    deps: &DepsMut,
    event_attributes: &mut Vec<Attribute>,
    mint_module_addr: &Addr,
    merge_msg: &MergeMsg,
    msgs: &mut Vec<WasmMsg>,
) -> Result<(), ContractError> {
    for burn_msg in &merge_msg.burn_ids {
        let collection_addr = StorageHelper::query_collection_address(
            &deps.querier,
            mint_module_addr,
            &burn_msg.collection_id,
        )?;

        let msg = KompleTokenModule(collection_addr).burn_msg(burn_msg.token_id.to_string())?;
        msgs.push(msg);

        event_attributes.push(MergeEventAttributes::new_burn_id_attribute(
            burn_msg.collection_id,
            burn_msg.token_id,
        ));
    }
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
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
