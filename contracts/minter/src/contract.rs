#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, ReplyOn, Response,
    StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use token::state::Locks;
use url::Url;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{CollectionInfo, Config, COLLECTION_INFO, CONFIG, TOKEN_ADDR};

use cw721_base::InstantiateMsg as TokenInstantiateMsg;
use token::msg::ExecuteMsg as TokenExecuteMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const TOKEN_INSTANTIATE_REPLY_ID: u64 = 1;
const MAX_DESCRIPTION_LENGTH: u32 = 512;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let whitelist = msg
        .whitelist
        .and_then(|w| deps.api.addr_validate(w.as_str()).ok());

    if msg.start_time.is_some() && env.block.time > msg.start_time.unwrap() {
        return Err(ContractError::InvalidStartTime {});
    }

    let config = Config {
        admin: info.sender.clone(),
        // TODO: Implement royalty
        royalty_info: None,
        per_address_limit: msg.per_address_limit,
        whitelist,
        start_time: msg.start_time,
    };
    CONFIG.save(deps.storage, &config)?;

    if msg.collection_info.description.len() > MAX_DESCRIPTION_LENGTH as usize {
        return Err(ContractError::DescriptionTooLong {});
    }

    Url::parse(&msg.collection_info.image)?;

    if let Some(ref external_link) = msg.collection_info.external_link {
        Url::parse(external_link)?;
    }

    let collection_info = CollectionInfo {
        name: msg.collection_info.name.clone(),
        description: msg.collection_info.description,
        image: msg.collection_info.image,
        external_link: msg.collection_info.external_link,
    };
    COLLECTION_INFO.save(deps.storage, &collection_info)?;

    // Instantiate token contract
    let msgs: Vec<SubMsg> = vec![SubMsg {
        msg: WasmMsg::Instantiate {
            code_id: msg.token_code_id,
            msg: to_binary(&TokenInstantiateMsg {
                name: msg.collection_info.name.clone(),
                symbol: msg.symbol,
                minter: env.contract.address.to_string(),
            })?,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: String::from("Framework wrapped token"),
        }
        .into(),
        id: TOKEN_INSTANTIATE_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    }];

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("minter", msg.minter)
        .add_attribute("collection_name", msg.collection_info.name)
        .add_submessages(msgs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateLocks { locks } => execute_update_locks(deps, env, info, locks),
        ExecuteMsg::Mint {
            recipient,
            token_id,
        } => unimplemented!(),
        ExecuteMsg::SetWhitelist { whitelist } => unimplemented!(),
        ExecuteMsg::UpdateStartTime(time) => unimplemented!(),
        ExecuteMsg::UpdatePerAddressLimit { per_address_limit } => unimplemented!(),
    }
}

pub fn execute_update_locks(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    locks: Locks,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let token_address = TOKEN_ADDR.load(deps.storage)?;

    let update_lock_msg: TokenExecuteMsg<Empty> = TokenExecuteMsg::UpdateLocks {
        locks: locks.clone(),
    };
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_address.to_string(),
        msg: to_binary(&update_lock_msg)?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "update_locks")
        .add_attribute("mint_lock", &locks.mint_lock.to_string())
        .add_attribute("burn_lock", &locks.burn_lock.to_string())
        .add_attribute("transfer_lock", &locks.transfer_lock.to_string())
        .add_attribute("send_lock", &locks.send_lock.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
    // match msg {
    //     QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
    // }
}
