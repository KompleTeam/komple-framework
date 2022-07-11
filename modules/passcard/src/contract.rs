#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Reply, ReplyOn,
    Response, StdResult, SubMsg, Timestamp, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

use token_contract::msg::{
    ExecuteMsg as TokenExecuteMsg, InstantiateMsg as TokenInstantiateMsg, TokenInfo,
};
use token_contract::state::CollectionInfo;

use crate::error::ContractError;
use crate::msg::{AddressResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, CONTROLLER_ADDR, MAIN_COLLECTIONS, PASSCARD_ADDR, PASSCARD_ID};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:passcard-module";
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

    let config = Config { admin };
    CONFIG.save(deps.storage, &config)?;

    PASSCARD_ID.save(deps.storage, &0)?;

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
        ExecuteMsg::CreatePasscard {
            code_id,
            collection_info,
            token_info,
            per_address_limit,
            start_time,
            whitelist,
            royalty,
            main_collections,
            max_token_limit,
        } => execute_create_passcard(
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
            main_collections,
            max_token_limit,
        ),
        ExecuteMsg::Mint { passcard_id } => execute_mint(deps, env, info, passcard_id),
    }
}

fn execute_create_passcard(
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
    main_collections: Vec<u32>,
    max_token_limit: Option<u32>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

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
                max_token_limit,
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

    let passcard_id = PASSCARD_ID.load(deps.storage)? + 1;

    PASSCARD_ID.save(deps.storage, &passcard_id)?;

    MAIN_COLLECTIONS.save(deps.storage, passcard_id, &main_collections)?;

    Ok(Response::new()
        .add_submessage(sub_msg)
        .add_attribute("action", "execute_create_passcard"))
}

fn execute_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    passcard_id: u32,
) -> Result<Response, ContractError> {
    // let config = CONFIG.load(deps.storage)?;
    // if config.mint_lock {
    //     return Err(ContractError::LockedMint {});
    // }

    let passcard_addr = PASSCARD_ADDR.load(deps.storage, passcard_id)?;

    let mint_msg = TokenExecuteMsg::Mint {
        owner: info.sender.to_string(),
    };
    let msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: passcard_addr.to_string(),
        msg: to_binary(&mint_msg)?,
        funds: info.funds,
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "execute_mint"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::PasscardAddress { passcard_id } => {
            to_binary(&query_passcard_address(deps, passcard_id)?)
        }
    }
}

fn query_passcard_address(deps: Deps, passcard_id: u32) -> StdResult<AddressResponse> {
    let addr = PASSCARD_ADDR.load(deps.storage, passcard_id)?;
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
            let passcard_id = PASSCARD_ID.load(deps.storage)?;
            PASSCARD_ADDR.save(
                deps.storage,
                passcard_id,
                &Addr::unchecked(res.contract_address),
            )?;
            Ok(Response::default().add_attribute("action", "instantiate_token_reply"))
        }
        Err(_) => Err(ContractError::TokenInstantiateError {}),
    }
}
