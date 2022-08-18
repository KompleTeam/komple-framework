#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Empty, Env,
    MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, Timestamp, WasmMsg, coin,
};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use cw_utils::parse_reply_instantiate_data;
use komple_types::metadata::Metadata as MetadataType;
use komple_types::query::ResponseWrapper;
use komple_types::tokens::Locks;
use komple_utils::{check_admin_privileges, check_funds};
use semver::Version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{
    BundleConfig, BundleInfo, Config, Contracts, BUNDLE_CONFIG, BUNDLE_INFO,
    CONFIG, CONTRACTS, LOCKS, MINTED_TOKENS_PER_ADDR, MINT_MODULE_ADDR, OPERATORS, TOKEN_IDS,
    TOKEN_LOCKS,
};

use cw721::{ContractInfoResponse, Cw721Execute};
use cw721_base::MintMsg;

use metadata_contract::msg::{
    ExecuteMsg as MetadataExecuteMsg, InstantiateMsg as MetadataInstantiateMsg,
};
use whitelist_contract::msg::{
    ConfigResponse as WhitelistConfigResponse, InstantiateMsg as WhitelistInstantiateMsg,
    QueryMsg as WhitelistQueryMsg,
};

pub type Cw721Contract<'a> = cw721_base::Cw721Contract<'a, Empty, Empty>;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:komple-token-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_DESCRIPTION_LENGTH: u32 = 512;

const METADATA_CONTRACT_INSTANTIATE_REPLY_ID: u64 = 1;
const WHITELIST_CONTRACT_INSTANTIATE_REPLY_ID: u64 = 2;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let locks = Locks {
        burn_lock: false,
        mint_lock: false,
        transfer_lock: false,
        send_lock: false,
    };

    let contracts = Contracts {
        whitelist: None,
        metadata: None,
    };
    CONTRACTS.save(deps.storage, &contracts)?;

    if msg.start_time.is_some() && env.block.time >= msg.start_time.unwrap() {
        return Err(ContractError::InvalidStartTime {});
    };

    if msg.max_token_limit.is_some() && msg.max_token_limit.unwrap() == 0 {
        return Err(ContractError::InvalidMaxTokenLimit {});
    };

    if msg.per_address_limit.is_some() && msg.per_address_limit.unwrap() == 0 {
        return Err(ContractError::InvalidPerAddressLimit {});
    };

    let bundle_config = BundleConfig {
        per_address_limit: msg.per_address_limit,
        start_time: msg.start_time,
        max_token_limit: msg.max_token_limit,
        unit_price: msg.unit_price,
    };
    BUNDLE_CONFIG.save(deps.storage, &bundle_config)?;

    if msg.royalty_share.is_some() && msg.royalty_share.unwrap() > Decimal::one() {
        return Err(ContractError::InvalidRoyaltyShare {});
    }

    let admin = deps.api.addr_validate(&msg.admin)?;
    let config = Config {
        admin,
        native_denom: msg.native_denom,
        royalty_share: msg.royalty_share,
    };
    CONFIG.save(deps.storage, &config)?;

    LOCKS.save(deps.storage, &locks)?;

    TOKEN_IDS.save(deps.storage, &0)?;

    MINT_MODULE_ADDR.save(deps.storage, &info.sender)?;

    if msg.bundle_info.description.len() > MAX_DESCRIPTION_LENGTH as usize {
        return Err(ContractError::DescriptionTooLong {});
    }

    let bundle_info = BundleInfo {
        bundle_type: msg.bundle_info.bundle_type,
        name: msg.bundle_info.name.clone(),
        description: msg.bundle_info.description,
        image: msg.bundle_info.image,
        external_link: msg.bundle_info.external_link,
    };
    BUNDLE_INFO.save(deps.storage, &bundle_info)?;

    let contract_info = ContractInfoResponse {
        name: msg.bundle_info.name.clone(),
        symbol: msg.token_info.symbol,
    };
    Cw721Contract::default()
        .contract_info
        .save(deps.storage, &contract_info)?;

    let minter = deps.api.addr_validate(&msg.token_info.minter)?;
    Cw721Contract::default()
        .minter
        .save(deps.storage, &minter)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("minter", msg.token_info.minter)
        .add_attribute("bundle_name", msg.bundle_info.name))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // LOCK MESSAGES
        ExecuteMsg::UpdateLocks { locks } => execute_update_locks(deps, env, info, locks),
        ExecuteMsg::UpdateTokenLock { token_id, locks } => {
            execute_update_token_locks(deps, env, info, token_id, locks)
        }

        // OPERATION MESSAGES
        ExecuteMsg::Mint { owner, metadata_id } => {
            execute_mint(deps, env, info, owner, metadata_id)
        }
        ExecuteMsg::Burn { token_id } => execute_burn(deps, env, info, token_id),
        ExecuteMsg::TransferNft {
            token_id,
            recipient,
        } => execute_transfer(deps, env, info, token_id, recipient),
        ExecuteMsg::SendNft {
            token_id,
            contract,
            msg,
        } => execute_send(deps, env, info, token_id, contract, msg),

        // CONFIG MESSAGES
        ExecuteMsg::UpdatePerAddressLimit { per_address_limit } => {
            execute_update_per_address_limit(deps, env, info, per_address_limit)
        }
        ExecuteMsg::UpdateStartTime { start_time } => {
            execute_update_start_time(deps, env, info, start_time)
        }

        // ADMIN MESSAGES
        ExecuteMsg::UpdateOperators { addrs } => execute_update_operators(deps, env, info, addrs),
        ExecuteMsg::AdminTransferNft {
            recipient,
            token_id,
        } => execute_admin_transfer(deps, env, info, token_id, recipient),
        ExecuteMsg::UpdateRoyaltyShare { royalty_share } => {
            execute_update_royalty_share(deps, env, info, royalty_share)
        }

        // CONTRACT MESSAGES
        ExecuteMsg::InitMetadataContract {
            code_id,
            metadata_type,
        } => execute_init_metadata_contract(deps, env, info, code_id, metadata_type),
        ExecuteMsg::InitWhitelistContract {
            code_id,
            instantiate_msg,
        } => execute_init_whitelist_contract(deps, env, info, code_id, instantiate_msg),

        // CW721 MESSAGES
        _ => {
            let res = Cw721Contract::default().execute(deps, env, info, msg.into());
            match res {
                Ok(res) => Ok(res),
                Err(e) => Err(e.into()),
            }
        }
    }
}

pub fn execute_update_operators(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    addrs: Vec<String>,
) -> Result<Response, ContractError> {
    let mint_module_addr = MINT_MODULE_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        mint_module_addr,
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

fn execute_update_royalty_share(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    royalty_share: Option<Decimal>,
) -> Result<Response, ContractError> {
    let mint_module_addr = MINT_MODULE_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let mut config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        mint_module_addr,
        operators,
    )?;

    if royalty_share.is_some() && royalty_share.unwrap() > Decimal::one() {
        return Err(ContractError::InvalidRoyaltyShare {});
    }

    config.royalty_share = royalty_share;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "execute_update_royalty_share"))
}

pub fn execute_update_locks(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    locks: Locks,
) -> Result<Response, ContractError> {
    let mint_module_addr = MINT_MODULE_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        mint_module_addr,
        operators,
    )?;

    LOCKS.save(deps.storage, &locks)?;

    Ok(Response::new()
        .add_attribute("action", "execute_update_locks")
        .add_attribute("mint_lock", locks.mint_lock.to_string())
        .add_attribute("burn_lock", locks.burn_lock.to_string())
        .add_attribute("transfer_lock", locks.transfer_lock.to_string())
        .add_attribute("send_lock", locks.send_lock.to_string()))
}

pub fn execute_update_token_locks(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    locks: Locks,
) -> Result<Response, ContractError> {
    let mint_module_addr = MINT_MODULE_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        mint_module_addr,
        operators,
    )?;

    if !Cw721Contract::default().tokens.has(deps.storage, &token_id) {
        return Err(ContractError::TokenNotFound {});
    }

    TOKEN_LOCKS.save(deps.storage, &token_id, &locks)?;

    Ok(Response::new()
        .add_attribute("action", "execute_update_token_locks")
        .add_attribute("token_id", token_id)
        .add_attribute("mint_lock", locks.mint_lock.to_string())
        .add_attribute("burn_lock", locks.burn_lock.to_string())
        .add_attribute("transfer_lock", locks.transfer_lock.to_string())
        .add_attribute("send_lock", locks.send_lock.to_string()))
}

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    metadata_id: Option<u32>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let bundle_config = BUNDLE_CONFIG.load(deps.storage)?;

    let locks = LOCKS.load(deps.storage)?;
    if locks.mint_lock {
        return Err(ContractError::MintLocked {});
    }

    let token_id = (TOKEN_IDS.load(deps.storage)?) + 1;

    let token_lock = TOKEN_LOCKS.may_load(deps.storage, &token_id.to_string())?;
    if token_lock.is_some() && token_lock.unwrap().mint_lock {
        return Err(ContractError::MintLocked {});
    }

    if bundle_config.max_token_limit.is_some()
        && token_id > bundle_config.max_token_limit.unwrap()
    {
        return Err(ContractError::TokenLimitReached {});
    }

    check_whitelist(&deps, &owner)?;

    let total_minted = MINTED_TOKENS_PER_ADDR
        .may_load(deps.storage, &owner)?
        .unwrap_or(0);

    if bundle_config.per_address_limit.is_some()
        && total_minted + 1 > bundle_config.per_address_limit.unwrap()
    {
        return Err(ContractError::TokenLimitReached {});
    }

    if bundle_config.start_time.is_some()
        && env.block.time < bundle_config.start_time.unwrap()
    {
        return Err(ContractError::MintingNotStarted {});
    }

    let mint_price = get_mint_price(&deps, &config, &bundle_config)?;
    if mint_price.is_some() {
        check_funds(
            &info,
            &config.native_denom,
            mint_price.as_ref().unwrap().amount,
        )?;
    }

    let mint_msg = MintMsg {
        token_id: token_id.to_string(),
        owner: owner.clone(),
        token_uri: None,
        extension: Empty {},
    };

    MINTED_TOKENS_PER_ADDR.save(deps.storage, &owner, &(total_minted + 1))?;
    TOKEN_IDS.save(deps.storage, &token_id)?;

    let contracts = CONTRACTS.load(deps.storage)?;
    if contracts.metadata.is_none() {
        return Err(ContractError::MetadataContractNotFound {});
    };

    let res = Cw721Contract::default().mint(deps, env, info, mint_msg);

    let mut msgs: Vec<CosmosMsg> = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contracts.metadata.unwrap().to_string(),
        msg: to_binary(&MetadataExecuteMsg::LinkMetadata {
            token_id,
            metadata_id,
        })
        .unwrap(),
        funds: vec![],
    })];
    if mint_price.is_some() {
        let payment_msg: CosmosMsg<Empty> = CosmosMsg::Bank(BankMsg::Send {
            to_address: config.admin.to_string(),
            amount: vec![mint_price.unwrap()],
        });
        msgs.push(payment_msg);
    }

    match res {
        Ok(res) => Ok(res.add_messages(msgs)),
        Err(e) => Err(e.into()),
    }
}

pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let locks = LOCKS.load(deps.storage)?;
    if locks.burn_lock {
        return Err(ContractError::BurnLocked {});
    }

    let token_lock = TOKEN_LOCKS.may_load(deps.storage, &token_id)?;
    if token_lock.is_some() && token_lock.unwrap().burn_lock {
        return Err(ContractError::BurnLocked {});
    }

    let contracts = CONTRACTS.load(deps.storage)?;
    if contracts.metadata.is_none() {
        return Err(ContractError::MetadataContractNotFound {});
    };

    let unlink_metadata_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contracts.metadata.unwrap().to_string(),
        msg: to_binary(&MetadataExecuteMsg::UnlinkMetadata {
            token_id: token_id.parse::<u32>().unwrap(),
        })
        .unwrap(),
        funds: vec![],
    });

    let res = Cw721Contract::default().burn(deps, env, info, token_id);

    match res {
        Ok(res) => Ok(res.add_message(unlink_metadata_msg)),
        Err(e) => Err(e.into()),
    }
}

pub fn execute_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    recipient: String,
) -> Result<Response, ContractError> {
    let locks = LOCKS.load(deps.storage)?;
    if locks.transfer_lock {
        return Err(ContractError::TransferLocked {});
    }

    let token_lock = TOKEN_LOCKS.may_load(deps.storage, &token_id)?;
    if token_lock.is_some() && token_lock.unwrap().transfer_lock {
        return Err(ContractError::TransferLocked {});
    }

    let res = Cw721Contract::default().transfer_nft(deps, env, info, recipient, token_id);
    match res {
        Ok(res) => Ok(res),
        Err(e) => Err(e.into()),
    }
}

pub fn execute_admin_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    recipient: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mint_module_addr = MINT_MODULE_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &&env.contract.address,
        &config.admin,
        mint_module_addr,
        operators,
    )?;

    let res = Cw721Contract::default().transfer_nft(deps, env, info, recipient, token_id);
    match res {
        Ok(res) => Ok(res),
        Err(e) => Err(e.into()),
    }
}

pub fn execute_send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    contract: String,
    msg: Binary,
) -> Result<Response, ContractError> {
    let locks = LOCKS.load(deps.storage)?;
    if locks.send_lock {
        return Err(ContractError::SendLocked {});
    }

    let token_lock = TOKEN_LOCKS.may_load(deps.storage, &token_id)?;
    if token_lock.is_some() && token_lock.unwrap().send_lock {
        return Err(ContractError::SendLocked {});
    }

    let res = Cw721Contract::default().send_nft(deps, env, info, contract, token_id, msg);
    match res {
        Ok(res) => Ok(res),
        Err(e) => Err(e.into()),
    }
}

pub fn execute_update_per_address_limit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    per_address_limit: Option<u32>,
) -> Result<Response, ContractError> {
    let mint_module_addr = MINT_MODULE_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        mint_module_addr,
        operators,
    )?;

    let mut bundle_config = BUNDLE_CONFIG.load(deps.storage)?;

    if per_address_limit.is_some() && per_address_limit.unwrap() == 0 {
        return Err(ContractError::InvalidPerAddressLimit {});
    }

    bundle_config.per_address_limit = per_address_limit;
    BUNDLE_CONFIG.save(deps.storage, &bundle_config)?;

    Ok(Response::new().add_attribute("action", "execute_update_per_address_limit"))
}

fn execute_update_start_time(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    start_time: Option<Timestamp>,
) -> Result<Response, ContractError> {
    let mint_module_addr = MINT_MODULE_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        mint_module_addr,
        operators,
    )?;

    let mut bundle_config = BUNDLE_CONFIG.load(deps.storage)?;

    if bundle_config.start_time.is_some()
        && env.block.time >= bundle_config.start_time.unwrap()
    {
        return Err(ContractError::AlreadyStarted {});
    }

    match start_time {
        Some(time) => {
            if env.block.time >= time {
                return Err(ContractError::InvalidStartTime {});
            }
            bundle_config.start_time = start_time;
        }
        None => bundle_config.start_time = None,
    }

    BUNDLE_CONFIG.save(deps.storage, &bundle_config)?;

    Ok(Response::new().add_attribute("action", "execute_update_start_time"))
}

fn execute_init_metadata_contract(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    code_id: u64,
    metadata_type: MetadataType,
) -> Result<Response, ContractError> {
    let mint_module_addr = MINT_MODULE_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        mint_module_addr,
        operators,
    )?;

    let sub_msg: SubMsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id,
            msg: to_binary(&MetadataInstantiateMsg {
                admin: config.admin.to_string(),
                metadata_type,
            })?,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: String::from("komple Framework Metadata Contract"),
        }
        .into(),
        id: METADATA_CONTRACT_INSTANTIATE_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    Ok(Response::new()
        .add_submessage(sub_msg)
        .add_attribute("action", "execute_init_metadata_contract"))
}

fn execute_init_whitelist_contract(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    code_id: u64,
    instantiate_msg: WhitelistInstantiateMsg,
) -> Result<Response, ContractError> {
    let mint_module_addr = MINT_MODULE_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        mint_module_addr,
        operators,
    )?;

    let sub_msg: SubMsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id,
            msg: to_binary(&instantiate_msg)?,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: String::from("komple Framework Whitelist Contract"),
        }
        .into(),
        id: WHITELIST_CONTRACT_INSTANTIATE_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    Ok(Response::new()
        .add_submessage(sub_msg)
        .add_attribute("action", "execute_init_whitelist_contract"))
}

fn check_whitelist(deps: &DepsMut, owner: &str) -> Result<(), ContractError> {
    let contracts = CONTRACTS.load(deps.storage)?;

    if contracts.whitelist.is_none() {
        return Ok(());
    }
    let whitelist = contracts.whitelist.unwrap();

    let whitelist_config: ResponseWrapper<WhitelistConfigResponse> = deps
        .querier
        .query_wasm_smart(whitelist.clone(), &WhitelistQueryMsg::Config {})?;
    if !whitelist_config.data.is_active {
        return Ok(());
    }

    let res: ResponseWrapper<bool> = deps.querier.query_wasm_smart(
        whitelist,
        &WhitelistQueryMsg::HasMember {
            member: owner.to_string(),
        },
    )?;
    if !res.data {
        return Err(ContractError::NotWhitelisted {});
    }

    let total_minted = MINTED_TOKENS_PER_ADDR
        .may_load(deps.storage, owner)?
        .unwrap_or(0);
    if total_minted + 1 > (whitelist_config.data.per_address_limit as u32) {
        return Err(ContractError::TokenLimitReached {});
    }

    Ok(())
}

fn get_mint_price(deps: &DepsMut, config: &Config, bundle_config: &BundleConfig) -> Result<Option<Coin>, ContractError> {
    let contracts = CONTRACTS.load(deps.storage)?;

    let bundle_price = bundle_config.unit_price.and_then(|price| Some(coin(price.u128(), &config.native_denom)));

    if contracts.whitelist.is_none() {
        return Ok(bundle_price);
    };

    let whitelist = contracts.whitelist.unwrap();

    let res: ResponseWrapper<WhitelistConfigResponse> = deps
        .querier
        .query_wasm_smart(whitelist.clone(), &WhitelistQueryMsg::Config {})?;

    if res.data.is_active {
        return Ok(Some(coin(res.data.unit_price.u128(), &config.native_denom)));
    } else {
        return Ok(bundle_price);
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Locks {} => to_binary(&query_locks(deps)?),
        QueryMsg::TokenLocks { token_id } => to_binary(&query_token_locks(deps, token_id)?),
        QueryMsg::MintedTokensPerAddress { address } => {
            to_binary(&query_minted_tokens_per_address(deps, address)?)
        }
        QueryMsg::BundleInfo {} => to_binary(&query_bundle_info(deps)?),
        QueryMsg::Contracts {} => to_binary(&query_contracts(deps)?),
        QueryMsg::ContractOperators {} => to_binary(&query_contract_operators(deps)?),
        _ => Cw721Contract::default().query(deps, env, msg.into()),
    }
}

fn query_config(deps: Deps) -> StdResult<ResponseWrapper<ConfigResponse>> {
    let config = CONFIG.load(deps.storage)?;
    let bundle_config = BUNDLE_CONFIG.load(deps.storage)?;
    Ok(ResponseWrapper::new(
        "locks",
        ConfigResponse {
            admin: config.admin.to_string(),
            native_denom: config.native_denom,
            per_address_limit: bundle_config.per_address_limit,
            start_time: bundle_config.start_time,
            max_token_limit: bundle_config.max_token_limit,
            unit_price: bundle_config.unit_price,
            royalty_share: config.royalty_share,
        },
    ))
}

fn query_locks(deps: Deps) -> StdResult<ResponseWrapper<Locks>> {
    let locks = LOCKS.load(deps.storage)?;
    Ok(ResponseWrapper::new("locks", locks))
}

fn query_token_locks(deps: Deps, token_id: String) -> StdResult<ResponseWrapper<Locks>> {
    let locks = TOKEN_LOCKS.load(deps.storage, &token_id)?;
    Ok(ResponseWrapper::new("locks", locks))
}

fn query_minted_tokens_per_address(deps: Deps, address: String) -> StdResult<ResponseWrapper<u32>> {
    let amount = MINTED_TOKENS_PER_ADDR
        .may_load(deps.storage, &address)?
        .unwrap_or(0);
    Ok(ResponseWrapper::new("minted_tokens_per_address", amount))
}

fn query_bundle_info(deps: Deps) -> StdResult<ResponseWrapper<BundleInfo>> {
    let bundle_info = BUNDLE_INFO.load(deps.storage)?;
    Ok(ResponseWrapper::new("bundle_info", bundle_info))
}

fn query_contracts(deps: Deps) -> StdResult<ResponseWrapper<Contracts>> {
    let contracts = CONTRACTS.load(deps.storage)?;
    Ok(ResponseWrapper::new("contracts", contracts))
}

fn query_contract_operators(deps: Deps) -> StdResult<ResponseWrapper<Vec<String>>> {
    let operators = OPERATORS.load(deps.storage).unwrap_or(vec![]);
    Ok(ResponseWrapper::new(
        "contract_operators",
        operators.iter().map(|o| o.to_string()).collect(),
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != METADATA_CONTRACT_INSTANTIATE_REPLY_ID
        && msg.id != WHITELIST_CONTRACT_INSTANTIATE_REPLY_ID
    {
        return Err(ContractError::InvalidReplyID {});
    }

    let reply = parse_reply_instantiate_data(msg.clone());
    match reply {
        Ok(res) => {
            let mut contracts = CONTRACTS.load(deps.storage)?;
            let contract: &str;
            match msg.id {
                METADATA_CONTRACT_INSTANTIATE_REPLY_ID => {
                    contracts.metadata = Some(Addr::unchecked(res.contract_address));
                    contract = "metadata";
                }
                WHITELIST_CONTRACT_INSTANTIATE_REPLY_ID => {
                    contracts.whitelist = Some(Addr::unchecked(res.contract_address));
                    contract = "whitelist";
                }
                _ => unreachable!(),
            }
            CONTRACTS.save(deps.storage, &contracts)?;
            Ok(Response::default()
                .add_attribute("action", format!("instantiate_{}_reply", contract)))
        }
        Err(_) => Err(ContractError::ContractsInstantiateError {}),
    }
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
