#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Empty};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{QueryMsg, ExecuteMsg};
use crate::state::{CONFIG, Locks};

use cw721::{ContractInfoResponse, Cw721Execute};
use cw721_base::{ContractError as Cw721ContractError, MintMsg, InstantiateMsg};

pub type Cw721Contract<'a> = cw721_base::Cw721Contract<'a, Empty, Empty>;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let contract_info = ContractInfoResponse {
        name: msg.name.clone(),
        symbol: msg.symbol
    };
    Cw721Contract::default().contract_info.save(deps.storage, &contract_info)?;

    let minter = deps.api.addr_validate(&msg.minter)?;
    Cw721Contract::default().minter.save(deps.storage, &minter)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("minter", msg.minter)
        .add_attribute("collection_name", msg.name))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<Empty>,
) -> Result<Response, Cw721ContractError> {
    match msg {
        ExecuteMsg::UpdateLocks { locks }
            => execute_update_locks(deps, env, info, locks),
        ExecuteMsg::Mint(mint_msg) => execute_mint(deps, env, info, mint_msg),
        ExecuteMsg::Burn { token_id } => execute_burn(deps, env, info, token_id),
        ExecuteMsg::TransferNft { token_id, recipient } => execute_transfer(deps, env, info, token_id, recipient),
        ExecuteMsg::SendNft { token_id, contract, msg } => execute_send(deps, env, info, token_id, contract, msg),
        _ => Cw721Contract::default().execute(deps, env, info, msg.into()),
    }
}

pub fn execute_update_locks(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    locks: Locks
) -> Result<Response, Cw721ContractError> {
    let mut config  = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {}.into());
    }

    config.locks = locks.clone();
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_locks")
        .add_attribute("mint_lock", locks.mint_lock.to_string())
        .add_attribute("burn_lock", locks.burn_lock.to_string())
        .add_attribute("transfer_lock", locks.transfer_lock.to_string())
        .add_attribute("send_lock", locks.send_lock.to_string())
    )
}

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mint_msg: MintMsg<Empty>
) -> Result<Response, Cw721ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if !config.locks.burn_lock {
        return Err(ContractError::MintLocked {}.into());
    }
    Cw721Contract::default().mint(deps, env, info, mint_msg)
}

pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, Cw721ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if !config.locks.burn_lock {
        return Err(ContractError::BurnedLocked {}.into());
    }
    Cw721Contract::default().burn(deps, env, info, token_id)
}

pub fn execute_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    recipient: String,
) -> Result<Response, Cw721ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if !config.locks.transfer_lock {
        return Err(ContractError::TransferLocked {}.into());
    }
    Cw721Contract::default().transfer_nft(deps, env, info, recipient, token_id)
}

pub fn execute_send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    contract: String,
    msg: Binary
) -> Result<Response, Cw721ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if !config.locks.send_lock {
        return Err(ContractError::SendLocked {}.into());
    }
    Cw721Contract::default().send_nft(deps, env, info, contract, token_id, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        _ => Cw721Contract::default().query(deps, env, msg.into()),
    }
}
