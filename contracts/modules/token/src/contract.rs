#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Attribute, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Reply,
    ReplyOn, Response, StdError, StdResult, SubMsg, Timestamp, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use cw_utils::parse_reply_instantiate_data;
use komple_types::collection::Collections;
use komple_types::metadata::Metadata as MetadataType;
use komple_types::query::ResponseWrapper;
use komple_types::token::{Locks, SubModules};
use komple_utils::check_admin_privileges;
use komple_utils::event::EventHelper;
use komple_whitelist_module::helper::KompleWhitelistHelper;
use semver::Version;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg as TokenExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg as TokenQueryMsg,
};
use crate::state::{
    Config, COLLECTION_TYPE, CONFIG, LOCKS, MINTED_TOKENS_PER_ADDR, MINT_MODULE_ADDR, OPERATORS,
    SUB_MODULES, TOKEN_IDS, TOKEN_LOCKS,
};

use cw721::ContractInfoResponse;
use cw721_base::{msg::ExecuteMsg as Cw721ExecuteMsg, MintMsg};

use komple_metadata_module::{helper::KompleMetadataModule, state::MetaInfo as MetadataMetaInfo};
use komple_whitelist_module::msg::InstantiateMsg as WhitelistInstantiateMsg;

pub type Cw721Contract<'a> =
    cw721_base::Cw721Contract<'a, Empty, Empty, TokenExecuteMsg, TokenQueryMsg>;
pub type ExecuteMsg = cw721_base::ExecuteMsg<Empty, TokenExecuteMsg>;
pub type QueryMsg = cw721_base::QueryMsg<TokenQueryMsg>;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:komple-token-module";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const METADATA_MODULE_INSTANTIATE_REPLY_ID: u64 = 1;
const WHITELIST_MODULE_INSTANTIATE_REPLY_ID: u64 = 2;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if msg.collection_config.start_time.is_some()
        && env.block.time >= msg.collection_config.start_time.unwrap()
    {
        return Err(ContractError::InvalidStartTime {});
    };
    if msg.collection_config.max_token_limit.is_some()
        && msg.collection_config.max_token_limit.unwrap() == 0
    {
        return Err(ContractError::InvalidMaxTokenLimit {});
    };
    if msg.collection_config.per_address_limit.is_some()
        && msg.collection_config.per_address_limit.unwrap() == 0
    {
        return Err(ContractError::InvalidPerAddressLimit {});
    };
    if msg.collection_type == Collections::Standard && msg.collection_config.ipfs_link.is_none() {
        return Err(ContractError::IpfsNotFound {});
    };

    if (msg.collection_type == Collections::Standard
        && msg.metadata_info.instantiate_msg.metadata_type != MetadataType::Standard)
        || (msg.collection_type != Collections::Standard
            && msg.metadata_info.instantiate_msg.metadata_type == MetadataType::Standard)
    {
        return Err(ContractError::InvalidCollectionMetadataType {});
    }

    COLLECTION_TYPE.save(deps.storage, &msg.collection_type)?;

    let admin = deps.api.addr_validate(&msg.admin)?;
    let creator = deps.api.addr_validate(&msg.creator)?;
    let config = Config {
        admin,
        creator,
        start_time: msg.collection_config.start_time,
        max_token_limit: msg.collection_config.max_token_limit,
        per_address_limit: msg.collection_config.per_address_limit,
        ipfs_link: msg.collection_config.ipfs_link,
    };
    CONFIG.save(deps.storage, &config)?;

    let locks = Locks {
        burn_lock: false,
        mint_lock: false,
        transfer_lock: false,
        send_lock: false,
    };
    LOCKS.save(deps.storage, &locks)?;

    TOKEN_IDS.save(deps.storage, &0)?;

    MINT_MODULE_ADDR.save(deps.storage, &info.sender)?;

    let sub_modules = SubModules {
        whitelist: None,
        metadata: None,
    };
    SUB_MODULES.save(deps.storage, &sub_modules)?;

    let contract_info = ContractInfoResponse {
        name: msg.collection_name.clone(),
        symbol: msg.token_info.symbol.clone(),
    };
    Cw721Contract::default()
        .contract_info
        .save(deps.storage, &contract_info)?;

    let minter = deps.api.addr_validate(&msg.token_info.minter)?;
    Cw721Contract::default()
        .minter
        .save(deps.storage, &minter)?;

    msg.metadata_info.instantiate_msg.admin = config.admin.to_string();
    let sub_msg: SubMsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id: msg.metadata_info.code_id,
            msg: to_binary(&msg.metadata_info.instantiate_msg)?,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: String::from("Komple Framework Metadata Module"),
        }
        .into(),
        id: METADATA_MODULE_INSTANTIATE_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    Ok(Response::new().add_submessage(sub_msg).add_event(
        EventHelper::new("komple_token_module")
            .add_attribute("action", "instantiate")
            .add_attribute("mint_module_addr", info.sender)
            .add_attribute("creator", config.creator)
            .add_attribute("minter", minter)
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
        ExecuteMsg::Extension { msg } => match msg {
            // LOCK MESSAGES
            TokenExecuteMsg::UpdateLocks { locks } => execute_update_locks(deps, env, info, locks),
            TokenExecuteMsg::UpdateTokenLocks { token_id, locks } => {
                execute_update_token_locks(deps, env, info, token_id, locks)
            }
            // OPERATION MESSAGES
            TokenExecuteMsg::Mint { owner, metadata_id } => {
                execute_mint(deps, env, info, owner, metadata_id)
            }
            TokenExecuteMsg::Burn { token_id } => execute_burn(deps, env, info, token_id),
            TokenExecuteMsg::TransferNft {
                token_id,
                recipient,
            } => execute_transfer(deps, env, info, token_id, recipient),
            TokenExecuteMsg::SendNft {
                token_id,
                contract,
                msg,
            } => execute_send(deps, env, info, token_id, contract, msg),
            // CONFIG MESSAGES
            TokenExecuteMsg::UpdatePerAddressLimit { per_address_limit } => {
                execute_update_per_address_limit(deps, env, info, per_address_limit)
            }
            TokenExecuteMsg::UpdateStartTime { start_time } => {
                execute_update_start_time(deps, env, info, start_time)
            }
            // ADMIN MESSAGES
            TokenExecuteMsg::UpdateModuleOperators { addrs } => {
                execute_update_module_operators(deps, env, info, addrs)
            }
            TokenExecuteMsg::AdminTransferNft {
                recipient,
                token_id,
            } => execute_admin_transfer(deps, env, info, token_id, recipient),
            // CONTRACT MESSAGES
            TokenExecuteMsg::InitWhitelistContract {
                code_id,
                instantiate_msg,
            } => execute_init_whitelist_module(deps, env, info, code_id, instantiate_msg),
        },
        _ => {
            match msg {
                // We are not allowing for normal mint endpoint
                Cw721ExecuteMsg::Mint(_mint_msg) => Err(ContractError::Unauthorized {}),
                Cw721ExecuteMsg::Burn { token_id } => execute_burn(deps, env, info, token_id),
                Cw721ExecuteMsg::SendNft {
                    token_id,
                    contract,
                    msg,
                } => execute_send(deps, env, info, token_id, contract, msg),
                Cw721ExecuteMsg::TransferNft {
                    token_id,
                    recipient,
                } => execute_transfer(deps, env, info, token_id, recipient),
                _ => {
                    let res = Cw721Contract::default().execute(deps, env, info, msg);
                    match res {
                        Ok(res) => Ok(res),
                        Err(e) => Err(e.into()),
                    }
                }
            }
        }
    }
}

pub fn execute_update_module_operators(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut addrs: Vec<String>,
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
        EventHelper::new("komple_token_module")
            .add_attribute("action", "update_module_operators")
            .add_attributes(event_attributes)
            .get(),
    ))
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

    Ok(Response::new().add_event(
        EventHelper::new("komple_token_module")
            .add_attribute("action", "update_locks")
            .add_attribute("mint_lock", locks.mint_lock.to_string())
            .add_attribute("burn_lock", locks.burn_lock.to_string())
            .add_attribute("transfer_lock", locks.transfer_lock.to_string())
            .add_attribute("send_lock", locks.send_lock.to_string())
            .get(),
    ))
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

    Ok(Response::new().add_event(
        EventHelper::new("komple_token_module")
            .add_attribute("action", "update_token_locks")
            .add_attribute("token_id", token_id)
            .add_attribute("mint_lock", locks.mint_lock.to_string())
            .add_attribute("burn_lock", locks.burn_lock.to_string())
            .add_attribute("transfer_lock", locks.transfer_lock.to_string())
            .add_attribute("send_lock", locks.send_lock.to_string())
            .get(),
    ))
}

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    metadata_id: Option<u32>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let collection_type = COLLECTION_TYPE.load(deps.storage)?;

    let total_minted = MINTED_TOKENS_PER_ADDR
        .may_load(deps.storage, &owner)?
        .unwrap_or(0);
    let token_id = (TOKEN_IDS.load(deps.storage)?) + 1;

    let locks = LOCKS.load(deps.storage)?;
    if locks.mint_lock {
        return Err(ContractError::MintLocked {});
    }

    let token_lock = TOKEN_LOCKS.may_load(deps.storage, &token_id.to_string())?;
    if token_lock.is_some() && token_lock.unwrap().mint_lock {
        return Err(ContractError::MintLocked {});
    }

    if config.max_token_limit.is_some() && token_id > config.max_token_limit.unwrap() {
        return Err(ContractError::TokenLimitReached {});
    }

    if config.per_address_limit.is_some() && total_minted + 1 > config.per_address_limit.unwrap() {
        return Err(ContractError::TokenLimitReached {});
    }

    if config.start_time.is_some() && env.block.time < config.start_time.unwrap() {
        return Err(ContractError::MintingNotStarted {});
    }

    // Whitelist checks
    let sub_modules = SUB_MODULES.load(deps.storage)?;
    if let Some(whitelist_addr) = sub_modules.whitelist {
        let whitelist_config =
            KompleWhitelistHelper::new(whitelist_addr).query_config(&deps.querier)?;

        if total_minted + 1 > (whitelist_config.per_address_limit as u32) {
            return Err(ContractError::TokenLimitReached {});
        }
    };

    let mint_msg = MintMsg {
        token_id: token_id.to_string(),
        owner: owner.clone(),
        token_uri: None,
        extension: Empty {},
    };

    MINTED_TOKENS_PER_ADDR.save(deps.storage, &owner, &(total_minted + 1))?;
    TOKEN_IDS.save(deps.storage, &token_id)?;

    let sub_modules = SUB_MODULES.load(deps.storage)?;
    if sub_modules.metadata.is_none() {
        return Err(ContractError::MetadataContractNotFound {});
    };

    let res = Cw721Contract::default().mint(deps, env, info, mint_msg);

    let mut msgs: Vec<CosmosMsg> = vec![];

    // If the collection is standard
    // Execute add_metadata message to save the ifps link to metadata module
    if collection_type == Collections::Standard {
        if config.ipfs_link.is_none() {
            return Err(ContractError::IpfsNotFound {});
        };

        let ifps_link = format!("{}/{}", config.ipfs_link.unwrap(), token_id);

        let msg = KompleMetadataModule(sub_modules.metadata.clone().unwrap()).add_metadata_msg(
            MetadataMetaInfo {
                image: Some(ifps_link),
                external_url: None,
                description: None,
                youtube_url: None,
                animation_url: None,
            },
            vec![],
        )?;
        msgs.push(msg.into())
    }
    // Link the metadata
    let msg = KompleMetadataModule(sub_modules.metadata.unwrap())
        .link_metadata_msg(token_id, metadata_id)?;
    msgs.push(msg.into());

    match res {
        Ok(res) => Ok(res.add_messages(msgs).add_event(
            EventHelper::new("komple_token_module")
                .add_attribute("action", "mint")
                .add_attribute("token_id", token_id.to_string())
                .add_attribute("owner", owner)
                .add_attribute("metadata_id", metadata_id.unwrap_or(0).to_string())
                .get(),
        )),
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

    let sub_modules = SUB_MODULES.load(deps.storage)?;
    if sub_modules.metadata.is_none() {
        return Err(ContractError::MetadataContractNotFound {});
    };

    let unlink_metadata_msg = KompleMetadataModule(sub_modules.metadata.unwrap())
        .unlink_metadata_msg(token_id.parse::<u32>().unwrap())?;

    let res = Cw721Contract::default().execute(
        deps,
        env,
        info,
        ExecuteMsg::Burn {
            token_id: token_id.clone(),
        },
    );
    match res {
        Ok(res) => Ok(res.add_message(unlink_metadata_msg).add_event(
            EventHelper::new("komple_token_module")
                .add_attribute("action", "burn")
                .add_attribute("token_id", token_id)
                .get(),
        )),
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

    let res = Cw721Contract::default().execute(
        deps,
        env,
        info,
        ExecuteMsg::TransferNft {
            recipient: recipient.clone(),
            token_id: token_id.clone(),
        },
    );
    match res {
        Ok(res) => Ok(res.add_event(
            EventHelper::new("komple_token_module")
                .add_attribute("action", "transfer")
                .add_attribute("token_id", token_id)
                .add_attribute("recipient", recipient)
                .get(),
        )),
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
        &env.contract.address,
        &config.admin,
        mint_module_addr,
        operators,
    )?;

    let res = Cw721Contract::default().execute(
        deps,
        env,
        info,
        ExecuteMsg::TransferNft {
            recipient: recipient.clone(),
            token_id: token_id.clone(),
        },
    );
    match res {
        Ok(res) => Ok(res.add_event(
            EventHelper::new("komple_token_module")
                .add_attribute("action", "admin_transfer")
                .add_attribute("token_id", token_id)
                .add_attribute("recipient", recipient)
                .get(),
        )),
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

    let res = Cw721Contract::default().execute(
        deps,
        env,
        info,
        ExecuteMsg::SendNft {
            contract: contract.clone(),
            token_id: contract.clone(),
            msg,
        },
    );
    match res {
        Ok(res) => Ok(res.add_event(
            EventHelper::new("komple_token_module")
                .add_attribute("action", "send")
                .add_attribute("token_id", token_id)
                .add_attribute("contract", contract)
                .get(),
        )),
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
    let mut config = CONFIG.load(deps.storage)?;

    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        mint_module_addr,
        operators,
    )?;

    if per_address_limit.is_some() && per_address_limit.unwrap() == 0 {
        return Err(ContractError::InvalidPerAddressLimit {});
    }

    config.per_address_limit = per_address_limit;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_event(
        EventHelper::new("komple_token_module")
            .add_attribute("action", "update_per_address_limit")
            .add_attribute(
                "per_address_limit",
                per_address_limit.unwrap_or(0).to_string(),
            )
            .get(),
    ))
}

fn execute_update_start_time(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    start_time: Option<Timestamp>,
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

    if config.start_time.is_some() && env.block.time >= config.start_time.unwrap() {
        return Err(ContractError::AlreadyStarted {});
    }

    match start_time {
        Some(time) => {
            if env.block.time >= time {
                return Err(ContractError::InvalidStartTime {});
            }
            config.start_time = start_time;
        }
        None => config.start_time = None,
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_event(
        EventHelper::new("komple_token_module")
            .add_attribute("action", "update_start_time")
            .add_attribute(
                "start_time",
                start_time.unwrap_or(Timestamp::from_seconds(0)).to_string(),
            )
            .get(),
    ))
}

fn execute_init_whitelist_module(
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
            label: String::from("Komple Framework Whitelist Module"),
        }
        .into(),
        id: WHITELIST_MODULE_INSTANTIATE_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    Ok(Response::new().add_submessage(sub_msg).add_event(
        EventHelper::new("komple_token_module")
            .add_attribute("action", "init_whitelist_module")
            .get(),
    ))
}

// fn get_mint_price(
//     deps: &DepsMut,
//     collection_config: &CollectionConfig,
// ) -> Result<Option<Coin>, ContractError> {
//     let sub_modules = SUB_MODULES.load(deps.storage)?;

//     let collection_price = collection_config
//         .unit_price
//         .map(|price| coin(price.u128(), &collection_config.native_denom));

//     if sub_modules.whitelist.is_none() {
//         return Ok(collection_price);
//     };

//     let whitelist = sub_modules.whitelist.unwrap();

//     let res: ResponseWrapper<WhitelistConfigResponse> = deps
//         .querier
//         .query_wasm_smart(whitelist, &WhitelistQueryMsg::Config {})?;

//     if res.data.is_active {
//         Ok(Some(coin(
//             res.data.unit_price.u128(),
//             &collection_config.native_denom,
//         )))
//     } else {
//         Ok(collection_price)
//     }
// }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Extension { msg } => match msg {
            TokenQueryMsg::Config {} => to_binary(&query_config(deps)?),
            TokenQueryMsg::Locks {} => to_binary(&query_locks(deps)?),
            TokenQueryMsg::TokenLocks { token_id } => {
                to_binary(&query_token_locks(deps, token_id)?)
            }
            TokenQueryMsg::MintedTokensPerAddress { address } => {
                to_binary(&query_minted_tokens_per_address(deps, address)?)
            }
            TokenQueryMsg::SubModules {} => to_binary(&query_sub_modules(deps)?),
            TokenQueryMsg::ModuleOperators {} => to_binary(&query_module_operators(deps)?),
        },
        _ => Cw721Contract::default().query(deps, env, msg),
    }
}

fn query_config(deps: Deps) -> StdResult<ResponseWrapper<Config>> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ResponseWrapper::new("locks", config))
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

fn query_sub_modules(deps: Deps) -> StdResult<ResponseWrapper<SubModules>> {
    let sub_modules = SUB_MODULES.load(deps.storage)?;
    Ok(ResponseWrapper::new("sub_modules", sub_modules))
}

fn query_module_operators(deps: Deps) -> StdResult<ResponseWrapper<Vec<String>>> {
    let operators = OPERATORS.load(deps.storage).unwrap_or_default();
    Ok(ResponseWrapper::new(
        "module_operators",
        operators.iter().map(|o| o.to_string()).collect(),
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != METADATA_MODULE_INSTANTIATE_REPLY_ID
        && msg.id != WHITELIST_MODULE_INSTANTIATE_REPLY_ID
    {
        return Err(ContractError::InvalidReplyID {});
    }

    let reply = parse_reply_instantiate_data(msg.clone());
    match reply {
        Ok(res) => {
            let mut sub_modules = SUB_MODULES.load(deps.storage)?;
            let sub_module: &str;
            match msg.id {
                METADATA_MODULE_INSTANTIATE_REPLY_ID => {
                    sub_modules.metadata = Some(Addr::unchecked(res.contract_address));
                    sub_module = "metadata";
                }
                WHITELIST_MODULE_INSTANTIATE_REPLY_ID => {
                    sub_modules.whitelist = Some(Addr::unchecked(res.contract_address));
                    sub_module = "whitelist";
                }
                _ => unreachable!(),
            }
            SUB_MODULES.save(deps.storage, &sub_modules)?;
            Ok(Response::default()
                .add_attribute("action", format!("instantiate_{}_reply", sub_module)))
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
