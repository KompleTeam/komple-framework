#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Addr, Attribute, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Order, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, Timestamp, Uint128,
    WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw_storage_plus::Bound;
use cw_utils::parse_reply_instantiate_data;

use komple_permission_module::msg::ExecuteMsg as PermissionExecuteMsg;
use komple_token_module::{
    helper::KompleTokenModule,
    msg::{InstantiateMsg as TokenInstantiateMsg, MetadataInfo, TokenInfo},
    state::CollectionConfig,
};
use komple_types::{
    fee::{FundInfo, MintFees},
    module::Modules,
    shared::RegisterMsg,
};
use komple_types::{query::ResponseWrapper, whitelist::WHITELIST_NAMESPACE};
use komple_utils::{
    check_admin_privileges,
    funds::check_cw20_fund_info,
    response::ResponseHelper,
    shared::{execute_lock_execute, execute_update_operators},
    storage::StorageHelper,
};
use komple_utils::{funds::check_single_coin, response::EventHelper};
use komple_whitelist_module::helper::KompleWhitelistHelper;
use semver::Version;

use crate::{error::ContractError, state::EXECUTE_LOCK};
use crate::{
    msg::CollectionFundInfo,
    state::{
        CollectionInfo, Config, BLACKLIST_COLLECTION_ADDRS, COLLECTION_ADDRS, COLLECTION_FUND_INFO,
        COLLECTION_ID, COLLECTION_INFO, CONFIG, HUB_ADDR, LINKED_COLLECTIONS, MINT_LOCKS,
        OPERATORS,
    },
};
use crate::{
    msg::{CollectionsResponse, ExecuteMsg, MigrateMsg, MintMsg, QueryMsg},
    state::CREATORS,
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
    msg: RegisterMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = deps.api.addr_validate(&msg.admin)?;

    let config = Config {
        admin,
        public_collection_creation: false,
        mint_lock: false,
    };
    CONFIG.save(deps.storage, &config)?;

    COLLECTION_ID.save(deps.storage, &0)?;

    HUB_ADDR.save(deps.storage, &info.sender)?;

    EXECUTE_LOCK.save(deps.storage, &false)?;

    Ok(ResponseHelper::new_module("mint", "instantiate").add_event(
        EventHelper::new("mint_instantiate")
            .add_attribute("admin", config.admin)
            .add_attribute(
                "public_collection_creation",
                config.public_collection_creation.to_string(),
            )
            .add_attribute("mint_lock", config.mint_lock.to_string())
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
    let execute_lock = EXECUTE_LOCK.load(deps.storage)?;
    if execute_lock {
        return Err(ContractError::ExecuteLocked {});
    };

    match msg {
        ExecuteMsg::CreateCollection {
            code_id,
            collection_config,
            collection_info,
            metadata_info,
            token_info,
            fund_info,
            linked_collections,
        } => execute_create_collection(
            deps,
            env,
            info,
            code_id,
            collection_config,
            collection_info,
            metadata_info,
            token_info,
            fund_info,
            linked_collections,
        ),
        ExecuteMsg::UpdatePublicCollectionCreation {
            public_collection_creation,
        } => execute_update_public_collection_creation(deps, env, info, public_collection_creation),
        ExecuteMsg::UpdateCollectionMintLock {
            collection_id,
            lock,
        } => execute_update_collection_mint_lock(deps, env, info, collection_id, lock),
        ExecuteMsg::Mint {
            collection_id,
            metadata_id,
        } => execute_mint(deps, env, info, collection_id, metadata_id),
        ExecuteMsg::AdminMint {
            collection_id,
            recipient,
            metadata_id,
        } => execute_admin_mint(deps, env, info, collection_id, recipient, metadata_id),
        ExecuteMsg::PermissionMint {
            permission_msg,
            mint_msg,
        } => execute_permission_mint(deps, env, info, permission_msg, mint_msg),
        ExecuteMsg::UpdateLinkedCollections {
            collection_id,
            linked_collections,
        } => execute_update_linked_collections(deps, env, info, collection_id, linked_collections),
        ExecuteMsg::UpdateCreators { addrs } => execute_update_creators(deps, env, info, addrs),
        ExecuteMsg::UpdateCollectionStatus {
            collection_id,
            is_blacklist,
        } => execute_update_collection_status(deps, env, info, collection_id, is_blacklist),
        ExecuteMsg::UpdateOperators { addrs } => {
            let config = CONFIG.load(deps.storage)?;
            let res = execute_update_operators(
                deps,
                info,
                "mint",
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
            let res = execute_lock_execute(deps, info, "mint", &env.contract.address, EXECUTE_LOCK);
            match res {
                Ok(res) => Ok(res),
                Err(err) => Err(err.into()),
            }
        }
    }
}

pub fn execute_create_collection(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    code_id: u64,
    collection_config: CollectionConfig,
    collection_info: CollectionInfo,
    metadata_info: MetadataInfo,
    mut token_info: TokenInfo,
    fund_info: CollectionFundInfo,
    linked_collections: Option<Vec<u32>>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // If public collection creation is enabled skip privilege checks
    if !config.public_collection_creation {
        let hub_addr = HUB_ADDR.may_load(deps.storage)?;
        let operators = OPERATORS.may_load(deps.storage)?;

        // Check for creators privilege
        let creators = CREATORS.may_load(deps.storage)?;
        match creators {
            Some(creators) => {
                // If the sender is not a creator
                if !creators.contains(&info.sender) {
                    check_admin_privileges(
                        &info.sender,
                        &env.contract.address,
                        &config.admin,
                        hub_addr,
                        operators,
                    )?;
                }
            }
            // If there are no creators
            None => {
                check_admin_privileges(
                    &info.sender,
                    &env.contract.address,
                    &config.admin,
                    hub_addr,
                    operators,
                )?;
            }
        }
    };

    token_info.minter = env.contract.address.to_string();
    let token_instantiate_msg = TokenInstantiateMsg {
        creator: info.sender.to_string(),
        collection_config: collection_config.clone(),
        collection_type: collection_info.clone().collection_type,
        collection_name: collection_info.clone().name,
        metadata_info,
        token_info,
    };
    let register_msg = RegisterMsg {
        admin: config.admin.to_string(),
        data: Some(to_binary(&token_instantiate_msg)?),
    };

    let contract_info = deps
        .querier
        .query_wasm_contract_info(env.contract.address)?;
    let sub_msg: SubMsg = SubMsg {
        msg: WasmMsg::Instantiate {
            code_id,
            msg: to_binary(&register_msg)?,
            funds: info.funds,
            admin: contract_info.admin,
            label: String::from("Komple Framework Token Module"),
        }
        .into(),
        id: TOKEN_INSTANTIATE_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    let collection_id = (COLLECTION_ID.load(deps.storage)?) + 1;

    if linked_collections.is_some() {
        check_collection_ids_exists(&deps, &linked_collections.clone().unwrap())?;
        LINKED_COLLECTIONS.save(deps.storage, collection_id, &linked_collections.unwrap())?;
    }

    COLLECTION_ID.save(deps.storage, &collection_id)?;

    COLLECTION_INFO.save(deps.storage, collection_id, &collection_info)?;

    let cw20_address = match fund_info.cw20_address {
        Some(addr) => Some(deps.api.addr_validate(&addr)?),
        None => None,
    };
    let fund_info = FundInfo {
        is_native: fund_info.is_native,
        denom: fund_info.denom,
        cw20_address,
    };

    if !fund_info.is_native {
        check_cw20_fund_info(&deps, &fund_info)?;
    };
    COLLECTION_FUND_INFO.save(deps.storage, collection_id, &fund_info)?;

    Ok(ResponseHelper::new_module("mint", "create_collection")
        .add_submessage(sub_msg)
        .add_event(
            EventHelper::new("mint_create_collection")
                .add_attribute("creator", token_instantiate_msg.creator)
                .add_attribute("minter", token_instantiate_msg.token_info.minter)
                .add_attribute("symbol", token_instantiate_msg.token_info.symbol)
                .add_attribute(
                    "collection_type",
                    token_instantiate_msg.collection_type.to_string(),
                )
                .add_attribute("collection_name", token_instantiate_msg.collection_name)
                .add_attribute("description", collection_info.description)
                .add_attribute("image", collection_info.image)
                .check_add_attribute(
                    &collection_info.external_link,
                    "external_link",
                    collection_info
                        .external_link
                        .as_ref()
                        .unwrap_or(&String::from("")),
                )
                .add_attribute("native_denom", collection_info.native_denom)
                .check_add_attribute(
                    &collection_config.start_time,
                    "start_time",
                    collection_config
                        .start_time
                        .unwrap_or(Timestamp::from_nanos(0))
                        .to_string(),
                )
                .check_add_attribute(
                    &collection_config.max_token_limit,
                    "max_token_limit",
                    collection_config.max_token_limit.unwrap_or(0).to_string(),
                )
                .check_add_attribute(
                    &collection_config.per_address_limit,
                    "per_address_limit",
                    collection_config.per_address_limit.unwrap_or(0).to_string(),
                )
                .check_add_attribute(
                    &collection_config.ipfs_link,
                    "ipfs_link",
                    collection_config
                        .ipfs_link
                        .as_ref()
                        .unwrap_or(&String::from("")),
                )
                .get(),
        ))
}

pub fn execute_update_public_collection_creation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    public_collection_creation: bool,
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

    config.public_collection_creation = public_collection_creation;
    CONFIG.save(deps.storage, &config)?;

    Ok(
        ResponseHelper::new_module("mint", "update_public_collection_creation").add_event(
            EventHelper::new("mint_update_public_collection_creation")
                .add_attribute(
                    "public_collection_creation",
                    public_collection_creation.to_string(),
                )
                .get(),
        ),
    )
}

pub fn execute_update_collection_mint_lock(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_id: u32,
    lock: bool,
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

    MINT_LOCKS.save(deps.storage, collection_id, &lock)?;

    Ok(
        ResponseHelper::new_module("mint", "update_mint_lock").add_event(
            EventHelper::new("mint_update_mint_lock")
                .add_attribute("collection_id", collection_id.to_string())
                .add_attribute("mint_lock", lock.to_string())
                .get(),
        ),
    )
}

fn execute_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_id: u32,
    metadata_id: Option<u32>,
) -> Result<Response, ContractError> {
    let hub_addr = HUB_ADDR.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    let mint_lock = MINT_LOCKS.may_load(deps.storage, collection_id)?;
    if let Some(_mint_lock) = mint_lock {
        return Err(ContractError::LockedMint {});
    }

    let mut msgs: Vec<CosmosMsg> = vec![];
    let mint_msg = MintMsg {
        collection_id,
        recipient: info.sender.to_string(),
        metadata_id,
    };

    let mut total_price = Uint128::zero();
    let mut is_whitelist = false;

    // Check for fee module address
    let fee_module_addr_res =
        StorageHelper::query_module_address(&deps.querier, &hub_addr, Modules::Fee.to_string());
    let fee_module_addr = match fee_module_addr_res {
        Ok(addr) => Some(addr),
        Err(_) => None,
    };

    // Get collection info
    let collection_info = COLLECTION_INFO.load(deps.storage, collection_id)?;

    // Get sub modules from collection
    let collection_addr = COLLECTION_ADDRS.load(deps.storage, collection_id)?;
    let sub_modules = StorageHelper::query_token_sub_modules(&deps.querier, &collection_addr)?;

    // Check for whitelist status
    if let Some(whitelist_addr) = sub_modules.whitelist {
        let res = KompleWhitelistHelper::new(whitelist_addr.clone()).query_is_active(&deps.querier);

        // Continue if whitelist is active
        if let Ok(is_active) = res {
            if is_active {
                // Query whitelist storage with owner address
                let query_key = StorageHelper::get_map_storage_key(
                    WHITELIST_NAMESPACE,
                    &[info.sender.as_bytes()],
                )?;
                let res = StorageHelper::query_storage::<bool>(
                    &deps.querier,
                    &whitelist_addr,
                    &query_key,
                )?;
                if res.is_none() {
                    return Err(ContractError::AddressNotWhitelisted {});
                }

                // If fee module is registered, check for whitelist minting price
                if fee_module_addr.is_some() {
                    // Whitelist is active and user is member
                    let res = StorageHelper::query_fixed_fee(
                        &deps.querier,
                        &fee_module_addr.as_ref().unwrap(),
                        Modules::Mint.to_string(),
                        MintFees::new_whitelist_price(collection_id),
                    );

                    let mut whitelist_price = Uint128::zero();

                    // If whitelist price exists
                    if let Ok(fixed_fee_response) = res {
                        whitelist_price = fixed_fee_response.value;
                    }
                    // Create send message if not zero
                    if !whitelist_price.is_zero() {
                        let msg = BankMsg::Send {
                            to_address: config.admin.to_string(),
                            amount: coins(
                                whitelist_price.u128(),
                                collection_info.native_denom.to_string(),
                            ),
                        };
                        msgs.push(msg.into());
                        total_price += whitelist_price;
                    }
                };

                is_whitelist = true;
            }
        }
    }

    // Standard collection mint flow
    if !is_whitelist {
        // If fee module is registered, check for standard minting price
        if fee_module_addr.is_some() {
            // Token mint price
            let res = StorageHelper::query_fixed_fee(
                &deps.querier,
                &fee_module_addr.unwrap(),
                Modules::Mint.to_string(),
                MintFees::new_price(collection_id),
            );
            if let Ok(fixed_fee_response) = res {
                let msg = BankMsg::Send {
                    to_address: config.admin.to_string(),
                    amount: coins(
                        fixed_fee_response.value.u128(),
                        collection_info.native_denom.to_string(),
                    ),
                };
                msgs.push(msg.into());
                total_price += fixed_fee_response.value;
            }
        }
    }

    if !total_price.is_zero() {
        check_single_coin(
            &info,
            Coin {
                denom: collection_info.native_denom,
                amount: total_price,
            },
        )?;
    };

    _execute_mint(deps, "mint", msgs, mint_msg)
}

fn execute_admin_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_id: u32,
    recipient: String,
    metadata_id: Option<u32>,
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

    let recipient = deps.api.addr_validate(&recipient)?;

    let msgs: Vec<CosmosMsg> = vec![];
    let mint_msg = MintMsg {
        collection_id,
        recipient: recipient.to_string(),
        metadata_id,
    };

    _execute_mint(deps, "admin_mint", msgs, mint_msg)
}

fn execute_permission_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    permission_msg: Binary,
    mint_msg: MintMsg,
) -> Result<Response, ContractError> {
    let hub_addr = HUB_ADDR.may_load(deps.storage)?;
    let operators = OPERATORS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    check_admin_privileges(
        &info.sender,
        &env.contract.address,
        &config.admin,
        hub_addr.clone(),
        operators,
    )?;

    let permission_module_addr = StorageHelper::query_module_address(
        &deps.querier,
        &hub_addr.unwrap(),
        Modules::Permission.to_string(),
    )?;

    let mut msgs: Vec<WasmMsg> = vec![];

    let permission_msg = PermissionExecuteMsg::Check {
        module: Modules::Mint.to_string(),
        msg: permission_msg,
    };
    msgs.push(WasmMsg::Execute {
        contract_addr: permission_module_addr.to_string(),
        msg: to_binary(&permission_msg)?,
        funds: info.funds.clone(),
    });

    msgs.push(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::AdminMint {
            collection_id: mint_msg.collection_id,
            recipient: mint_msg.recipient.clone(),
            metadata_id: mint_msg.metadata_id,
        })?,
        funds: info.funds,
    });

    Ok(ResponseHelper::new_module("mint", "permission_mint")
        .add_messages(msgs)
        .add_event(
            EventHelper::new("mint_permission_mint")
                .add_attribute("collection_id", mint_msg.collection_id.to_string())
                .add_attribute("recipient", mint_msg.recipient)
                .check_add_attribute(
                    &mint_msg.metadata_id,
                    "metadata_id",
                    mint_msg.metadata_id.as_ref().unwrap_or(&0).to_string(),
                )
                .get(),
        ))
}

fn _execute_mint(
    deps: DepsMut,
    action: &str,
    mut msgs: Vec<CosmosMsg>,
    mint_msg: MintMsg,
) -> Result<Response, ContractError> {
    let collection_addr = COLLECTION_ADDRS.load(deps.storage, mint_msg.collection_id)?;

    let msg = KompleTokenModule(collection_addr)
        .mint_msg(mint_msg.recipient.clone(), mint_msg.metadata_id)?;
    msgs.push(msg.into());

    Ok(ResponseHelper::new_module("mint", action)
        .add_messages(msgs)
        .add_event(
            EventHelper::new(format!("mint_{}", action))
                .add_attribute("recipient", mint_msg.recipient)
                .add_attribute("collection_id", mint_msg.collection_id.to_string())
                .check_add_attribute(
                    &mint_msg.metadata_id,
                    "metadata_id",
                    mint_msg.metadata_id.as_ref().unwrap_or(&0).to_string(),
                )
                .get(),
        ))
}

fn execute_update_linked_collections(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_id: u32,
    linked_collections: Vec<u32>,
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

    if linked_collections.contains(&collection_id) {
        return Err(ContractError::SelfLinkedCollection {});
    };

    let mut ids_to_check = vec![collection_id];
    ids_to_check.extend(&linked_collections);
    check_collection_ids_exists(&deps, &ids_to_check)?;

    LINKED_COLLECTIONS.save(deps.storage, collection_id, &linked_collections)?;

    Ok(Response::new()
        .add_attribute("name", "komple_framework")
        .add_attribute("module", "mint")
        .add_attribute("action", "update_linked_collections")
        .add_event(EventHelper::new("mint_update_linked_collections").get()))
}

fn execute_update_collection_status(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_id: u32,
    is_blacklist: bool,
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

    match is_blacklist {
        // Blacklist case
        true => {
            if BLACKLIST_COLLECTION_ADDRS.has(deps.storage, collection_id) {
                return Err(ContractError::AlreadyBlacklisted {});
            };

            let collection_addr = COLLECTION_ADDRS.may_load(deps.storage, collection_id)?;
            if collection_addr.is_none() {
                return Err(ContractError::CollectionIdNotFound {});
            };

            COLLECTION_ADDRS.remove(deps.storage, collection_id);
            BLACKLIST_COLLECTION_ADDRS.save(
                deps.storage,
                collection_id,
                &collection_addr.unwrap(),
            )?;
        }
        // Whitelist case
        false => {
            if COLLECTION_ADDRS.has(deps.storage, collection_id) {
                return Err(ContractError::AlreadyWhitelistlisted {});
            };

            let collection_addr =
                BLACKLIST_COLLECTION_ADDRS.may_load(deps.storage, collection_id)?;
            if collection_addr.is_none() {
                return Err(ContractError::CollectionIdNotFound {});
            };

            BLACKLIST_COLLECTION_ADDRS.remove(deps.storage, collection_id);
            COLLECTION_ADDRS.save(deps.storage, collection_id, &collection_addr.unwrap())?;
        }
    }

    Ok(Response::new()
        .add_attribute("name", "komple_framework")
        .add_attribute("module", "mint")
        .add_attribute("action", "update_collection_status")
        .add_event(
            EventHelper::new("mint_update_collection_status")
                .add_attribute("collection_id", collection_id.to_string())
                .add_attribute("is_blacklist", is_blacklist.to_string())
                .get(),
        ))
}

fn execute_update_creators(
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

    CREATORS.save(deps.storage, &addrs)?;

    Ok(
        ResponseHelper::new_module("mint", "update_creators").add_event(
            EventHelper::new("mint_update_creators")
                .add_attributes(event_attributes)
                .get(),
        ),
    )
}

fn check_collection_ids_exists(
    deps: &DepsMut,
    collection_ids: &Vec<u32>,
) -> Result<(), ContractError> {
    for collection_id in collection_ids {
        if !COLLECTION_ADDRS.has(deps.storage, *collection_id) {
            return Err(ContractError::CollectionIdNotFound {});
        }
    }
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::CollectionAddress(collection_id) => {
            to_binary(&query_collection_address(deps, collection_id)?)
        }
        QueryMsg::CollectionInfo { collection_id } => {
            to_binary(&query_collection_info(deps, collection_id)?)
        }
        QueryMsg::Operators {} => to_binary(&query_operators(deps)?),
        QueryMsg::LinkedCollections { collection_id } => {
            to_binary(&query_linked_collections(deps, collection_id)?)
        }
        QueryMsg::Collections {
            blacklist,
            start_after,
            limit,
        } => to_binary(&query_collections(deps, blacklist, start_after, limit)?),
        QueryMsg::Creators {} => to_binary(&query_creators(deps)?),
        QueryMsg::MintLock { collection_id } => to_binary(&query_mint_lock(deps, collection_id)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ResponseWrapper<Config>> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ResponseWrapper::new("config", config))
}

fn query_collection_address(deps: Deps, collection_id: u32) -> StdResult<ResponseWrapper<String>> {
    let addr = COLLECTION_ADDRS.load(deps.storage, collection_id)?;
    Ok(ResponseWrapper::new("collection_address", addr.to_string()))
}

fn query_collection_info(
    deps: Deps,
    collection_id: u32,
) -> StdResult<ResponseWrapper<CollectionInfo>> {
    let collection_info = COLLECTION_INFO.load(deps.storage, collection_id)?;
    Ok(ResponseWrapper::new("collection_info", collection_info))
}

fn query_operators(deps: Deps) -> StdResult<ResponseWrapper<Vec<String>>> {
    let addrs = OPERATORS.may_load(deps.storage)?;
    let addrs = match addrs {
        Some(addrs) => addrs.iter().map(|a| a.to_string()).collect(),
        None => vec![],
    };
    Ok(ResponseWrapper::new("operators", addrs))
}

fn query_linked_collections(
    deps: Deps,
    collection_id: u32,
) -> StdResult<ResponseWrapper<Vec<u32>>> {
    let linked_collection_ids = LINKED_COLLECTIONS.may_load(deps.storage, collection_id)?;
    let linked_collection_ids = match linked_collection_ids {
        Some(linked_collection_ids) => linked_collection_ids,
        None => vec![],
    };
    Ok(ResponseWrapper::new(
        "linked_collections",
        linked_collection_ids,
    ))
}

fn query_collections(
    deps: Deps,
    blacklist: bool,
    start_after: Option<u32>,
    limit: Option<u8>,
) -> StdResult<ResponseWrapper<Vec<CollectionsResponse>>> {
    let limit = limit.unwrap_or(30) as usize;
    let start = start_after.map(Bound::exclusive);

    let collections_state = match blacklist {
        true => BLACKLIST_COLLECTION_ADDRS,
        false => COLLECTION_ADDRS,
    };

    let collections = collections_state
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (collection_id, address) = item.unwrap();
            CollectionsResponse {
                collection_id,
                address: address.to_string(),
            }
        })
        .collect::<Vec<CollectionsResponse>>();

    Ok(ResponseWrapper::new("collections", collections))
}

fn query_creators(deps: Deps) -> StdResult<ResponseWrapper<Vec<String>>> {
    let addrs = CREATORS.may_load(deps.storage)?;
    let addrs = match addrs {
        Some(addrs) => addrs.iter().map(|a| a.to_string()).collect(),
        None => vec![],
    };
    Ok(ResponseWrapper::new("creators", addrs))
}

fn query_mint_lock(deps: Deps, collection_id: u32) -> StdResult<ResponseWrapper<Option<bool>>> {
    let mint_lock = MINT_LOCKS.may_load(deps.storage, collection_id)?;
    Ok(ResponseWrapper::new("mint_lock", mint_lock))
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
            COLLECTION_ADDRS.save(
                deps.storage,
                collection_id,
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
