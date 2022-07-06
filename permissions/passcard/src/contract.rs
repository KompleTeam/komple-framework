#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, ListPasscardsResponse, QueryMsg};
use crate::state::{
    Config, Passcard, PasscardInfo, CONFIG, MINTABLE_PASSCARDS, PASSCARDS, PASSCARD_AMOUNT,
    PASSCARD_INFO,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:passcard";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let controller_addr = deps.api.addr_validate(&msg.controller_address)?;

    let config = Config {
        admin: info.sender,
        main_collection: None,
        controller_addr,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddPasscard {
            collection_id,
            base_price,
            passcard_info,
        } => execute_add_passcard(deps, _env, info, collection_id, base_price, passcard_info),
    }
}

pub fn execute_add_passcard(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_id: u32,
    base_price: Uint128,
    passcard_info: PasscardInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    };

    let passcard_amount = (PASSCARD_AMOUNT.may_load(deps.storage, collection_id)?).unwrap_or(0);

    for passcard_id in (passcard_amount + 1)..=(passcard_amount + passcard_info.total_num) {
        let passcard = Passcard {
            id: passcard_id,
            price: base_price,
            on_sale: true,
            owner: config.admin.clone(),
        };
        PASSCARDS.save(deps.storage, (collection_id, passcard_id), &passcard)?;
        MINTABLE_PASSCARDS.save(deps.storage, (collection_id, passcard_id), &true)?;
    }
    PASSCARD_INFO.save(deps.storage, collection_id, &passcard_info)?;
    PASSCARD_AMOUNT.save(
        deps.storage,
        collection_id,
        &(passcard_amount + passcard_info.total_num),
    )?;

    Ok(Response::default()
        .add_attribute("action", "add_sub_collection")
        .add_attribute("id", collection_id.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ListAvailablePasscards { collection_id } => {
            to_binary(&query_available_passcards(deps, collection_id)?)
        }
        QueryMsg::GetPasscard {
            collection_id,
            passcard_id,
        } => to_binary(&query_passcard(deps, collection_id, passcard_id)?),
    }
}

fn query_available_passcards(deps: Deps, collection_id: u32) -> StdResult<ListPasscardsResponse> {
    let passcard_info = PASSCARD_INFO.load(deps.storage, collection_id)?;

    // TODO: Implement pagination to this query
    let passcards = PASSCARDS
        .range(deps.storage, None, None, Order::Ascending)
        .filter(|item| {
            if let Ok(((sub_col_id, _), info)) = item {
                return sub_col_id == &collection_id && info.on_sale;
            } else {
                return false;
            }
        })
        .map(|item| {
            let ((_, _), info) = item.unwrap();

            Ok(info)
        })
        .collect::<StdResult<_>>()?;

    let response = ListPasscardsResponse {
        passcards,
        total_num: passcard_info.total_num,
    };

    Ok(response)
}

fn query_passcard(deps: Deps, collection_id: u32, passcard_id: u16) -> StdResult<Passcard> {
    let passcard = PASSCARDS.load(deps.storage, (collection_id, passcard_id))?;
    Ok(passcard)
}
