use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MergeMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG, HUB_ADDR, OPERATORS};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Attribute, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use komple_mint_module::helper::KompleMintModule;
use komple_permission_module::msg::ExecuteMsg as PermissionExecuteMsg;
use komple_token_module::helper::KompleTokenModule;
use komple_types::module::Modules;
use komple_types::query::ResponseWrapper;
use komple_utils::event::EventHelper;
use komple_utils::{check_admin_privileges, storage::StorageHelper};
use semver::Version;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:komple-merge-module";
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
        merge_lock: false,
    };
    CONFIG.save(deps.storage, &config)?;

    HUB_ADDR.save(deps.storage, &info.sender)?;

    Ok(Response::new().add_event(
        EventHelper::new("komple_merge_module")
            .add_attribute("action", "instantiate")
            .add_attribute("admin", config.admin)
            .add_attribute("hub_addr", info.sender)
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
        ExecuteMsg::UpdateMergeLock { lock } => execute_update_merge_lock(deps, env, info, lock),
        ExecuteMsg::Merge { msg } => execute_merge(deps, env, info, msg),
        ExecuteMsg::PermissionMerge {
            permission_msg,
            merge_msg,
        } => execute_permission_merge(deps, env, info, permission_msg, merge_msg),
        ExecuteMsg::UpdateOperators { addrs } => execute_update_operators(deps, env, info, addrs),
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

    Ok(Response::new().add_event(
        EventHelper::new("komple_merge_module")
            .add_attribute("action", "update_merge_lock")
            .add_attribute("lock", lock.to_string())
            .get(),
    ))
}

fn execute_merge(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: Binary,
) -> Result<Response, ContractError> {
    let mut msgs: Vec<WasmMsg> = vec![];

    let mut event_attributes: Vec<Attribute> = vec![];

    make_merge_msg(&deps, &info, &mut event_attributes, msg, &mut msgs)?;

    Ok(Response::new().add_messages(msgs).add_event(
        EventHelper::new("komple_merge_module")
            .add_attribute("action", "merge")
            .add_attributes(event_attributes)
            .get(),
    ))
}

fn execute_permission_merge(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    permission_msg: Binary,
    merge_msg: Binary,
) -> Result<Response, ContractError> {
    // TODO: Should only be callable by admin

    let hub_addr = HUB_ADDR.load(deps.storage)?;
    let permission_module_addr =
        StorageHelper::query_module_address(&deps.querier, &hub_addr, Modules::Permission)?;

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

    Ok(Response::new().add_messages(msgs).add_event(
        EventHelper::new("komple_merge_module")
            .add_attribute("action", "permission_merge")
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
        EventHelper::new("komple_merge_module")
            .add_attribute("action".to_string(), "update_operators".to_string())
            .add_attributes(event_attributes)
            .get(),
    ))
}

/// Constructs the mint and burn messages
fn make_merge_msg(
    deps: &DepsMut,
    info: &MessageInfo,
    event_attributes: &mut Vec<Attribute>,
    msg: Binary,
    msgs: &mut Vec<WasmMsg>,
) -> Result<(), ContractError> {
    let hub_addr = HUB_ADDR.load(deps.storage)?;
    let mint_module_addr =
        StorageHelper::query_module_address(&deps.querier, &hub_addr, Modules::Mint)?;

    // MergeMsg contains mint_id, burn_ids and metadata_id
    let merge_msg: MergeMsg = from_binary(&msg)?;

    // Throw an error if there are no burn messages
    if merge_msg.burn_ids.is_empty() {
        return Err(ContractError::BurnNotFound {});
    }

    // Pushes the burn_ids inside msgs list
    make_burn_messages(deps, event_attributes, &mint_module_addr, &merge_msg, msgs)?;

    let msg = KompleMintModule(mint_module_addr).mint_to_msg(
        info.sender.to_string(),
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

fn make_burn_messages(
    deps: &DepsMut,
    event_attributes: &mut Vec<Attribute>,
    mint_module_addr: &Addr,
    merge_msg: &MergeMsg,
    msgs: &mut Vec<WasmMsg>,
) -> Result<(), ContractError> {
    for (index, burn_msg) in merge_msg.burn_ids.iter().enumerate() {
        let collection_addr = StorageHelper::query_collection_address(
            &deps.querier,
            mint_module_addr,
            &burn_msg.collection_id,
        )?;

        let lock_msg =
            KompleTokenModule(collection_addr).burn_msg(burn_msg.token_id.to_string())?;
        msgs.push(lock_msg);

        event_attributes.push(Attribute::new(
            format!("burn_msg/{}", index),
            format!("token_id/{}", burn_msg.token_id),
        ));
        event_attributes.push(Attribute::new(
            format!("burn_msg/{}", index),
            format!("collection_id/{}", burn_msg.collection_id),
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
