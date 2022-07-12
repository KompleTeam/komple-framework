#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Reply, ReplyOn,
    Response, StdResult, SubMsg, Timestamp, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

use rift_types::module::Modules;
use rift_types::query::{AddressResponse, ControllerQueryMsg};
use rift_utils::{check_admin_privileges, get_module_address};
use token_contract::msg::{
    ExecuteMsg as TokenExecuteMsg, InstantiateMsg as TokenInstantiateMsg, TokenInfo,
};
use token_contract::state::CollectionInfo;

use permission_module::msg::ExecuteMsg as PermissionExecuteMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, COLLECTION_ID, CONFIG, CONTROLLER_ADDR, TOKEN_ADDRS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:mint-module";
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
            collection_info,
            token_info,
            per_address_limit,
            start_time,
            whitelist,
            royalty,
        } => execute_create_collection(
            deps,
            env,
            info,
            code_id,
            collection_info,
            token_info,
            per_address_limit,
            start_time,
            whitelist,
            royalty,
        ),
        ExecuteMsg::UpdateMintLock { lock } => execute_update_mint_lock(deps, env, info, lock),
        ExecuteMsg::Mint { collection_id } => execute_mint(deps, env, info, collection_id),
        ExecuteMsg::MintTo {
            collection_id,
            recipient,
        } => execute_mint_to(deps, env, info, collection_id, recipient),
        ExecuteMsg::PermissionMint {
            permission_msg,
            mint_msg,
        } => execute_permission_mint(deps, env, info, permission_msg, mint_msg),
    }
}

pub fn execute_create_collection(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    code_id: u64,
    collection_info: CollectionInfo,
    token_info: TokenInfo,
    per_address_limit: Option<u32>,
    start_time: Option<Timestamp>,
    whitelist: Option<String>,
    royalty: Option<String>,
) -> Result<Response, ContractError> {
    let controller_addr = CONTROLLER_ADDR.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(&info.sender, &config.admin, Some(&controller_addr), None)?;

    // Instantiate token contract
    let sub_msg: SubMsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id,
            msg: to_binary(&TokenInstantiateMsg {
                admin: config.admin.to_string(),
                token_info,
                collection_info,
                per_address_limit,
                start_time,
                whitelist,
                royalty,
                max_token_limit: None,
            })?,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: String::from("Rift Framework Token Contract"),
        }
        .into(),
        id: TOKEN_INSTANTIATE_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    COLLECTION_ID.update(deps.storage, |value| -> StdResult<u32> { Ok(value + 1) })?;

    Ok(Response::new()
        .add_submessage(sub_msg)
        .add_attribute("action", "create_collection"))
}

pub fn execute_update_mint_lock(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    lock: bool,
) -> Result<Response, ContractError> {
    let controller_addr = CONTROLLER_ADDR.load(deps.storage)?;
    let mut config = CONFIG.load(deps.storage)?;

    check_admin_privileges(&info.sender, &config.admin, Some(&controller_addr), None)?;

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
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.mint_lock {
        return Err(ContractError::LockedMint {});
    }

    _execute_mint(
        deps,
        info.clone(),
        "mint",
        collection_id,
        info.sender.to_string(),
    )
}

fn execute_mint_to(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_id: u32,
    recipient: String,
) -> Result<Response, ContractError> {
    let controller_addr = CONTROLLER_ADDR.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(&info.sender, &config.admin, Some(&controller_addr), None)?;

    let owner = deps.api.addr_validate(&recipient)?;

    _execute_mint(deps, info, "mint_to", collection_id, owner.to_string())
}

fn execute_permission_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    permission_msg: Binary,
    _mint_msg: Binary,
) -> Result<Response, ContractError> {
    let controller_addr = CONTROLLER_ADDR.load(deps.storage)?;
    let permission_module_addr =
        get_module_address(&deps, &controller_addr, Modules::PermissionModule)?;

    let mut msgs: Vec<CosmosMsg> = vec![];

    let permission_msg = PermissionExecuteMsg::Check {
        module: Modules::MintModule,
        msg: permission_msg,
    };
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: permission_module_addr.to_string(),
        msg: to_binary(&permission_msg)?,
        funds: info.funds,
    }));

    // TODO: Construct mint msg and send

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "execute_permission_mint"))
}

fn _execute_mint(
    deps: DepsMut,
    info: MessageInfo,
    action: &str,
    collection_id: u32,
    owner: String,
) -> Result<Response, ContractError> {
    let token_address = TOKEN_ADDRS.load(deps.storage, collection_id)?;

    let mint_msg = TokenExecuteMsg::Mint { owner };
    let msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_address.to_string(),
        msg: to_binary(&mint_msg)?,
        funds: info.funds,
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", action))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::CollectionAddress(collection_id) => {
            to_binary(&query_collection_address(deps, collection_id)?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

fn query_collection_address(deps: Deps, collection_id: u32) -> StdResult<AddressResponse> {
    let addr = TOKEN_ADDRS.load(deps.storage, collection_id)?;
    Ok(AddressResponse {
        address: addr.to_string(),
    })
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
            TOKEN_ADDRS.save(
                deps.storage,
                collection_id,
                &Addr::unchecked(res.contract_address),
            )?;
            Ok(Response::default().add_attribute("action", "instantiate_token_reply"))
        }
        Err(_) => Err(ContractError::TokenInstantiateError {}),
    }
}
