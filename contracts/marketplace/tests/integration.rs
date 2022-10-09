use cosmwasm_std::{to_binary, Addr, Coin, Decimal, Empty, Uint128};
use cw721_base::msg::{ExecuteMsg as Cw721ExecuteMsg, QueryMsg as Cw721QueryMsg};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_fee_module::{
    msg::{
        ExecuteMsg as FeeModuleExecuteMsg, InstantiateMsg as FeeModuleInstantiateMsg,
        QueryMsg as FeeModuleQueryMsg,
    },
    state::PercentagePayment as FeeModulePercentagePayment,
};
use komple_hub_module::{
    msg::{
        ExecuteMsg as HubExecuteMsg, InstantiateMsg as HubInstantiateMsg, QueryMsg as HubQueryMsg,
    },
    state::HubInfo,
};
use komple_marketplace_module::msg::{ExecuteMsg, InstantiateMsg};
use komple_mint_module::msg::{ExecuteMsg as MintExecuteMsg, InstantiateMsg as MintInstantiateMsg};
use komple_token_module::{
    msg::{
        ExecuteMsg as TokenExecuteMsg, InstantiateMsg as TokenInstantiateMsg,
        QueryMsg as TokenQueryMsg, TokenInfo,
    },
    state::{CollectionConfig, CollectionInfo, Contracts},
};
use komple_types::collection::Collections;
use komple_types::fee::Fees;
use komple_types::metadata::Metadata as MetadataType;
use komple_types::module::Modules;
use komple_types::query::ResponseWrapper;
use komple_utils::query_collection_address;
use std::str::FromStr;

pub const CREATOR: &str = "juno..creator";
pub const USER: &str = "juno..user";
pub const RANDOM: &str = "juno..random";
pub const ADMIN: &str = "juno..admin";
pub const RANDOM_2: &str = "juno..random2";
pub const NATIVE_DENOM: &str = "native_denom";
pub const TEST_DENOM: &str = "test_denom";

pub fn hub_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_hub_module::contract::execute,
        komple_hub_module::contract::instantiate,
        komple_hub_module::contract::query,
    )
    .with_reply(komple_hub_module::contract::reply);
    Box::new(contract)
}

pub fn mint_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_mint_module::contract::execute,
        komple_mint_module::contract::instantiate,
        komple_mint_module::contract::query,
    )
    .with_reply(komple_mint_module::contract::reply);
    Box::new(contract)
}

pub fn token_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_token_module::contract::execute,
        komple_token_module::contract::instantiate,
        komple_token_module::contract::query,
    )
    .with_reply(komple_token_module::contract::reply);
    Box::new(contract)
}

pub fn marketplace_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_marketplace_module::contract::execute,
        komple_marketplace_module::contract::instantiate,
        komple_marketplace_module::contract::query,
    );
    Box::new(contract)
}

pub fn metadata_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_metadata_module::contract::execute,
        komple_metadata_module::contract::instantiate,
        komple_metadata_module::contract::query,
    );
    Box::new(contract)
}

pub fn fee_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_fee_module::contract::execute,
        komple_fee_module::contract::instantiate,
        komple_fee_module::contract::query,
    );
    Box::new(contract)
}

pub fn mock_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(USER),
                vec![Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(1_000_000),
                }],
            )
            .unwrap();
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(RANDOM),
                vec![Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(1_000_000),
                }],
            )
            .unwrap();
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(RANDOM_2),
                vec![Coin {
                    denom: TEST_DENOM.to_string(),
                    amount: Uint128::new(1_000_000),
                }],
            )
            .unwrap();
    })
}

fn setup_fee_module(app: &mut App, fee_module_addr: &Addr) {
    // Komple is 4%
    let msg = FeeModuleExecuteMsg::SetFee {
        fee_type: Fees::Percentage,
        module_name: Modules::Marketplace.to_string(),
        fee_name: "komple".to_string(),
        data: to_binary(&FeeModulePercentagePayment {
            address: Some("contract0".to_string()),
            value: Decimal::from_str("0.04").unwrap(),
        })
        .unwrap(),
    };
    let _ = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            fee_module_addr.clone(),
            &msg,
            &vec![],
        )
        .unwrap();
    // Community pool is 2%
    let msg = FeeModuleExecuteMsg::SetFee {
        fee_type: Fees::Percentage,
        module_name: Modules::Marketplace.to_string(),
        fee_name: "community".to_string(),
        data: to_binary(&FeeModulePercentagePayment {
            address: Some("juno..community".to_string()),
            value: Decimal::from_str("0.02").unwrap(),
        })
        .unwrap(),
    };
    let _ = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            fee_module_addr.clone(),
            &msg,
            &vec![],
        )
        .unwrap();
    // Hub owner is 2%
    let msg = FeeModuleExecuteMsg::SetFee {
        fee_type: Fees::Percentage,
        module_name: Modules::Marketplace.to_string(),
        fee_name: "hub_admin".to_string(),
        data: to_binary(&FeeModulePercentagePayment {
            address: None,
            value: Decimal::from_str("0.02").unwrap(),
        })
        .unwrap(),
    };
    let _ = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            fee_module_addr.clone(),
            &msg,
            &vec![],
        )
        .unwrap();

    let msg = FeeModuleQueryMsg::TotalPercentageFees {
        module_name: Modules::Marketplace.to_string(),
    };
    let _: ResponseWrapper<Decimal> = app
        .wrap()
        .query_wasm_smart(fee_module_addr.clone(), &msg)
        .unwrap();
}

fn set_royalties(app: &mut App, fee_module_addr: &Addr, collection_id: &u32, royalty: &str) {
    let msg = FeeModuleExecuteMsg::SetFee {
        fee_type: Fees::Percentage,
        module_name: Modules::Mint.to_string(),
        fee_name: format!("collection_{}_royalty", collection_id.to_string()),
        data: to_binary(&FeeModulePercentagePayment {
            address: None,
            value: Decimal::from_str(royalty).unwrap(),
        })
        .unwrap(),
    };
    let _ = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            fee_module_addr.clone(),
            &msg,
            &vec![],
        )
        .unwrap();
}

fn setup_hub_module(app: &mut App, is_marbu: bool) -> Addr {
    let hub_code_id = app.store_code(hub_module());

    let fee_module_addr = match is_marbu {
        true => {
            let fee_code_id = app.store_code(fee_module());

            let msg = FeeModuleInstantiateMsg {
                admin: ADMIN.to_string(),
            };
            let fee_module_addr = app
                .instantiate_contract(
                    fee_code_id,
                    Addr::unchecked(ADMIN),
                    &msg,
                    &vec![],
                    "test",
                    None,
                )
                .unwrap();

            setup_fee_module(app, &fee_module_addr);

            Some(fee_module_addr.to_string())
        }
        false => None,
    };

    let msg = HubInstantiateMsg {
        admin: None,
        hub_info: HubInfo {
            name: "Test Hub".to_string(),
            description: "Test Hub".to_string(),
            image: "https://example.com/image.png".to_string(),
            external_link: None,
        },
        marbu_fee_module: fee_module_addr,
    };
    let hub_addr = app
        .instantiate_contract(hub_code_id, Addr::unchecked(ADMIN), &msg, &[], "test", None)
        .unwrap();

    hub_addr
}

fn setup_modules(app: &mut App, hub_addr: Addr) -> (Addr, Addr) {
    let mint_code_id = app.store_code(mint_module());
    let marketplace_code_id = app.store_code(marketplace_module());

    let instantiate_msg = to_binary(&MintInstantiateMsg {
        admin: ADMIN.to_string(),
    })
    .unwrap();
    let msg = HubExecuteMsg::RegisterModule {
        module: Modules::Mint.to_string(),
        msg: instantiate_msg,
        code_id: mint_code_id,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &vec![])
        .unwrap();
    let instantiate_msg = to_binary(&InstantiateMsg {
        admin: ADMIN.to_string(),
        native_denom: NATIVE_DENOM.to_string(),
    })
    .unwrap();
    let msg = HubExecuteMsg::RegisterModule {
        module: Modules::Marketplace.to_string(),
        msg: instantiate_msg,
        code_id: marketplace_code_id,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &vec![])
        .unwrap();

    let msg = HubQueryMsg::ModuleAddress {
        module: Modules::Mint.to_string(),
    };
    let mint_res: ResponseWrapper<Addr> =
        app.wrap().query_wasm_smart(hub_addr.clone(), &msg).unwrap();
    let msg = HubQueryMsg::ModuleAddress {
        module: Modules::Marketplace.to_string(),
    };
    let marketplace_res: ResponseWrapper<Addr> =
        app.wrap().query_wasm_smart(hub_addr.clone(), &msg).unwrap();

    (mint_res.data, marketplace_res.data)
}

pub fn create_collection(
    app: &mut App,
    mint_module_addr: Addr,
    creator_addr: &str,
    token_module_code_id: u64,
) {
    let collection_info = CollectionInfo {
        collection_type: Collections::Standard,
        name: "Test Collection".to_string(),
        description: "Test Collection".to_string(),
        image: "https://image.com".to_string(),
        external_link: None,
    };
    let token_info = TokenInfo {
        symbol: "TEST".to_string(),
        minter: mint_module_addr.to_string(),
    };
    let collection_config = CollectionConfig {
        per_address_limit: None,
        start_time: None,
        unit_price: None,
        native_denom: NATIVE_DENOM.to_string(),
        max_token_limit: None,
        ipfs_link: Some("some-link".to_string()),
    };
    let msg = MintExecuteMsg::CreateCollection {
        code_id: token_module_code_id,
        token_instantiate_msg: TokenInstantiateMsg {
            admin: "".to_string(),
            creator: "".to_string(),
            collection_info,
            collection_config,
            token_info,
            royalty_share: None,
        },
        linked_collections: None,
    };
    let _ = app
        .execute_contract(
            Addr::unchecked(creator_addr),
            mint_module_addr,
            &msg,
            &vec![],
        )
        .unwrap();
}

pub fn setup_metadata_module(
    app: &mut App,
    token_module_addr: Addr,
    metadata_type: MetadataType,
) -> Addr {
    let metadata_code_id = app.store_code(metadata_module());

    let msg: Cw721ExecuteMsg<Empty, TokenExecuteMsg> = Cw721ExecuteMsg::Extension {
        msg: TokenExecuteMsg::InitMetadataContract {
            code_id: metadata_code_id,
            metadata_type,
        },
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
        .unwrap();

    let res: ResponseWrapper<Contracts> = app
        .wrap()
        .query_wasm_smart(
            token_module_addr.clone(),
            &Cw721QueryMsg::Extension {
                msg: TokenQueryMsg::Contracts {},
            },
        )
        .unwrap();
    res.data.metadata.unwrap()
}

pub fn mint_token(app: &mut App, mint_module_addr: Addr, collection_id: u32, sender: &str) {
    let msg = MintExecuteMsg::Mint {
        collection_id,
        metadata_id: None,
    };
    let _ = app
        .execute_contract(Addr::unchecked(sender), mint_module_addr, &msg, &vec![])
        .unwrap();
}

pub fn setup_token_module_operators(app: &mut App, token_module_addr: Addr, addrs: Vec<String>) {
    let msg: Cw721ExecuteMsg<Empty, TokenExecuteMsg> = Cw721ExecuteMsg::Extension {
        msg: TokenExecuteMsg::UpdateOperators { addrs },
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), token_module_addr, &msg, &vec![])
        .unwrap();
}

pub fn give_approval_to_module(
    app: &mut App,
    token_module_addr: Addr,
    owner: &str,
    operator_addr: &Addr,
) {
    let msg: Cw721ExecuteMsg<Empty, TokenExecuteMsg> = Cw721ExecuteMsg::ApproveAll {
        operator: operator_addr.to_string(),
        expires: None,
    };
    let _ = app
        .execute_contract(Addr::unchecked(owner), token_module_addr, &msg, &vec![])
        .unwrap();
}

pub fn setup_marketplace_listing(
    app: &mut App,
    mint_module_addr: &Addr,
    marketplace_module_addr: &Addr,
    collection_id: u32,
    token_id: u32,
    price: Uint128,
) {
    let collection_addr =
        query_collection_address(&app.wrap(), &mint_module_addr, &collection_id).unwrap();

    setup_token_module_operators(
        app,
        collection_addr.clone(),
        vec![marketplace_module_addr.to_string()],
    );

    let msg = ExecuteMsg::ListFixedToken {
        collection_id,
        token_id,
        price,
    };
    let _ = app
        .execute_contract(
            Addr::unchecked(USER),
            marketplace_module_addr.clone(),
            &msg,
            &vec![],
        )
        .unwrap();
}

mod initialization {
    use super::*;

    use komple_types::module::Modules;

    use komple_hub_module::ContractError;
    use komple_utils::query_module_address;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let hub_addr = setup_hub_module(&mut app, false);
        let marketplace_module_code_id = app.store_code(marketplace_module());

        let instantiate_msg = to_binary(&InstantiateMsg {
            admin: ADMIN.to_string(),
            native_denom: NATIVE_DENOM.to_string(),
        })
        .unwrap();
        let msg = HubExecuteMsg::RegisterModule {
            module: Modules::Marketplace.to_string(),
            msg: instantiate_msg,
            code_id: marketplace_module_code_id,
        };
        let _ = app.execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &vec![]);

        let res = query_module_address(&app.wrap(), &hub_addr, Modules::Marketplace).unwrap();
        assert_eq!(res, "contract1")
    }

    #[test]
    fn test_happy_path_with_fee_module() {
        let mut app = mock_app();
        let hub_addr = setup_hub_module(&mut app, false);
        let marketplace_module_code_id = app.store_code(marketplace_module());
        let fee_module_code_id = app.store_code(fee_module());

        let instantiate_msg = to_binary(&FeeModuleInstantiateMsg {
            admin: ADMIN.to_string(),
        })
        .unwrap();
        let msg = HubExecuteMsg::RegisterModule {
            module: Modules::Fee.to_string(),
            msg: instantiate_msg,
            code_id: fee_module_code_id,
        };
        let _ = app.execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &vec![]);

        let instantiate_msg = to_binary(&InstantiateMsg {
            admin: ADMIN.to_string(),
            native_denom: NATIVE_DENOM.to_string(),
        })
        .unwrap();
        let msg = HubExecuteMsg::RegisterModule {
            module: Modules::Marketplace.to_string(),
            msg: instantiate_msg,
            code_id: marketplace_module_code_id,
        };
        let _ = app.execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &vec![]);

        let res = query_module_address(&app.wrap(), &hub_addr, Modules::Marketplace).unwrap();
        assert_eq!(res, "contract2")
    }

    #[test]
    fn test_invalid_sender() {
        let mut app = mock_app();
        let hub_addr = setup_hub_module(&mut app, false);
        let marketplace_module_code_id = app.store_code(marketplace_module());

        let instantiate_msg = to_binary(&InstantiateMsg {
            admin: ADMIN.to_string(),
            native_denom: NATIVE_DENOM.to_string(),
        })
        .unwrap();
        let msg = HubExecuteMsg::RegisterModule {
            module: Modules::Marketplace.to_string(),
            msg: instantiate_msg,
            code_id: marketplace_module_code_id,
        };
        let err = app
            .execute_contract(Addr::unchecked(USER), hub_addr.clone(), &msg, &vec![])
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            ContractError::Unauthorized {}.to_string()
        )
    }
}

mod actions {
    use super::*;

    use cosmwasm_std::Uint128;
    use komple_marketplace_module::{
        msg::{ExecuteMsg as MarketplaceExecuteMsg, QueryMsg as MarketplaceQueryMsg},
        ContractError as MarketplaceContractError,
    };
    use komple_token_module::msg::ExecuteMsg as TokenExecuteMsg;
    use komple_token_module::ContractError as TokenContractError;

    use komple_types::metadata::Metadata;

    mod listing {
        use super::*;

        mod fixed_tokens {
            use super::*;

            use komple_marketplace_module::state::FixedListing;
            use komple_types::{metadata::Metadata, query::ResponseWrapper, tokens::Locks};
            use komple_utils::{query_collection_address, query_token_locks};

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let hub_addr = setup_hub_module(&mut app, false);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, hub_addr.clone());

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                setup_metadata_module(&mut app, collection_addr.clone(), Metadata::Standard);

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                setup_token_module_operators(
                    &mut app,
                    collection_addr.clone(),
                    vec![marketplace_module_addr.to_string()],
                );

                let msg = MarketplaceExecuteMsg::ListFixedToken {
                    collection_id: 1,
                    token_id: 1,
                    price: Uint128::new(1_000_000),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        marketplace_module_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = MarketplaceQueryMsg::FixedListing {
                    collection_id: 1,
                    token_id: 1,
                };
                let res: ResponseWrapper<FixedListing> = app
                    .wrap()
                    .query_wasm_smart(marketplace_module_addr, &msg)
                    .unwrap();
                assert_eq!(res.data.owner, USER.to_string());
                assert_eq!(res.data.price, Uint128::new(1_000_000));

                let locks = query_token_locks(&app.wrap(), &collection_addr, &1).unwrap();
                assert_eq!(locks.transfer_lock, true);
                assert_eq!(locks.send_lock, true);
                assert_eq!(locks.burn_lock, true);
            }

            #[test]
            fn test_invalid_owner() {
                let mut app = mock_app();
                let hub_addr = setup_hub_module(&mut app, false);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, hub_addr.clone());

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                setup_metadata_module(&mut app, collection_addr.clone(), Metadata::Standard);

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                let msg = MarketplaceExecuteMsg::ListFixedToken {
                    collection_id: 1,
                    token_id: 1,
                    price: Uint128::new(1_000_000),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(RANDOM),
                        marketplace_module_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    MarketplaceContractError::Unauthorized {}.to_string()
                );
            }

            #[test]
            fn test_invalid_locks() {
                let mut app = mock_app();
                let hub_addr = setup_hub_module(&mut app, false);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, hub_addr.clone());

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                setup_metadata_module(&mut app, collection_addr.clone(), Metadata::Standard);

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                let listing_msg = MarketplaceExecuteMsg::ListFixedToken {
                    collection_id: 1,
                    token_id: 1,
                    price: Uint128::new(1_000_000),
                };

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                let unlock = Locks {
                    mint_lock: false,
                    burn_lock: false,
                    transfer_lock: false,
                    send_lock: true,
                };
                let transfer_lock = Locks {
                    mint_lock: false,
                    burn_lock: false,
                    transfer_lock: true,
                    send_lock: true,
                };
                let msg: Cw721ExecuteMsg<Empty, TokenExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: TokenExecuteMsg::UpdateTokenLock {
                        token_id: "1".to_string(),
                        locks: transfer_lock.clone(),
                    },
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        collection_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        marketplace_module_addr.clone(),
                        &listing_msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string().to_string(),
                    TokenContractError::TransferLocked {}.to_string()
                );

                let msg: Cw721ExecuteMsg<Empty, TokenExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: TokenExecuteMsg::UpdateTokenLock {
                        token_id: "1".to_string(),
                        locks: unlock.clone(),
                    },
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        collection_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg: Cw721ExecuteMsg<Empty, TokenExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: TokenExecuteMsg::UpdateLocks {
                        locks: transfer_lock.clone(),
                    },
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        collection_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        marketplace_module_addr.clone(),
                        &listing_msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string().to_string(),
                    TokenContractError::TransferLocked {}.to_string()
                );

                let msg: Cw721ExecuteMsg<Empty, TokenExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: TokenExecuteMsg::UpdateLocks {
                        locks: unlock.clone(),
                    },
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        collection_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();
            }

            #[test]
            fn test_invalid_operator() {
                let mut app = mock_app();
                let hub_addr = setup_hub_module(&mut app, false);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, hub_addr.clone());

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                setup_metadata_module(&mut app, collection_addr.clone(), Metadata::Standard);

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                let msg = MarketplaceExecuteMsg::ListFixedToken {
                    collection_id: 1,
                    token_id: 1,
                    price: Uint128::new(1_000_000),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        marketplace_module_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().source().unwrap().to_string(),
                    TokenContractError::Unauthorized {}.to_string()
                );
            }
        }
    }

    mod delisting {
        use super::*;

        use cosmwasm_std::Empty;
        use komple_utils::query_collection_address;

        mod fixed_tokens {
            use komple_utils::query_token_locks;

            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let hub_addr = setup_hub_module(&mut app, false);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, hub_addr.clone());

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                setup_metadata_module(&mut app, collection_addr.clone(), Metadata::Standard);

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                setup_token_module_operators(
                    &mut app,
                    collection_addr.clone(),
                    vec![marketplace_module_addr.to_string()],
                );

                let msg = MarketplaceExecuteMsg::ListFixedToken {
                    collection_id: 1,
                    token_id: 1,
                    price: Uint128::new(1_000_000),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        marketplace_module_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let locks = query_token_locks(&app.wrap(), &collection_addr, &1).unwrap();
                assert_eq!(locks.transfer_lock, true);
                assert_eq!(locks.send_lock, true);
                assert_eq!(locks.burn_lock, true);

                let msg = MarketplaceExecuteMsg::DelistFixedToken {
                    collection_id: 1,
                    token_id: 1,
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        marketplace_module_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let locks = query_token_locks(&app.wrap(), &collection_addr, &1).unwrap();
                assert_eq!(locks.transfer_lock, false);
                assert_eq!(locks.send_lock, false);
                assert_eq!(locks.burn_lock, false);

                let msg = MarketplaceQueryMsg::FixedListing {
                    collection_id: 1,
                    token_id: 1,
                };
                let res: Result<Empty, cosmwasm_std::StdError> =
                    app.wrap().query_wasm_smart(marketplace_module_addr, &msg);
                assert!(res.is_err());
            }

            #[test]
            fn test_invalid_owner() {
                let mut app = mock_app();
                let hub_addr = setup_hub_module(&mut app, false);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, hub_addr.clone());

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                setup_metadata_module(&mut app, collection_addr.clone(), Metadata::Standard);

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                setup_token_module_operators(
                    &mut app,
                    collection_addr.clone(),
                    vec![marketplace_module_addr.to_string()],
                );

                let msg = MarketplaceExecuteMsg::ListFixedToken {
                    collection_id: 1,
                    token_id: 1,
                    price: Uint128::new(1_000_000),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        marketplace_module_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = MarketplaceExecuteMsg::DelistFixedToken {
                    collection_id: 1,
                    token_id: 1,
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(RANDOM),
                        marketplace_module_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    MarketplaceContractError::Unauthorized {}.to_string()
                )
            }

            #[test]
            fn test_invalid_operator() {
                let mut app = mock_app();
                let hub_addr = setup_hub_module(&mut app, false);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, hub_addr.clone());

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                setup_metadata_module(&mut app, collection_addr.clone(), Metadata::Standard);

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                setup_token_module_operators(
                    &mut app,
                    collection_addr.clone(),
                    vec![marketplace_module_addr.to_string()],
                );

                let msg = MarketplaceExecuteMsg::ListFixedToken {
                    collection_id: 1,
                    token_id: 1,
                    price: Uint128::new(1_000_000),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        marketplace_module_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                setup_token_module_operators(&mut app, collection_addr.clone(), vec![]);

                let msg = MarketplaceExecuteMsg::DelistFixedToken {
                    collection_id: 1,
                    token_id: 1,
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        marketplace_module_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().source().unwrap().to_string(),
                    TokenContractError::Unauthorized {}.to_string()
                )
            }
        }
    }

    mod pricing {
        use komple_marketplace_module::state::FixedListing;
        use komple_types::{marketplace::Listing, query::ResponseWrapper};

        use super::*;

        mod fixed_tokens {
            use komple_utils::query_collection_address;

            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let hub_addr = setup_hub_module(&mut app, false);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, hub_addr.clone());

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                setup_metadata_module(&mut app, collection_addr.clone(), Metadata::Standard);

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                setup_marketplace_listing(
                    &mut app,
                    &mint_module_addr,
                    &marketplace_module_addr,
                    1,
                    1,
                    Uint128::new(1_000_000),
                );

                let msg = MarketplaceExecuteMsg::UpdatePrice {
                    listing_type: Listing::Fixed,
                    collection_id: 1,
                    token_id: 1,
                    price: Uint128::new(200_000_000),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        marketplace_module_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = MarketplaceQueryMsg::FixedListing {
                    collection_id: 1,
                    token_id: 1,
                };
                let res: ResponseWrapper<FixedListing> = app
                    .wrap()
                    .query_wasm_smart(marketplace_module_addr, &msg)
                    .unwrap();
                assert_eq!(res.data.owner, USER.to_string());
                assert_eq!(res.data.price, Uint128::new(200_000_000));
            }

            #[test]
            fn test_invalid_owner() {
                let mut app = mock_app();
                let hub_addr = setup_hub_module(&mut app, false);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, hub_addr.clone());

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                setup_metadata_module(&mut app, collection_addr.clone(), Metadata::Standard);

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                setup_marketplace_listing(
                    &mut app,
                    &mint_module_addr,
                    &marketplace_module_addr,
                    1,
                    1,
                    Uint128::new(1_000_000),
                );

                let msg = MarketplaceExecuteMsg::UpdatePrice {
                    listing_type: Listing::Fixed,
                    collection_id: 1,
                    token_id: 1,
                    price: Uint128::new(200_000_000),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(RANDOM),
                        marketplace_module_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    MarketplaceContractError::Unauthorized {}.to_string()
                )
            }
        }
    }

    mod buying {
        use super::*;

        use cosmwasm_std::coin;
        use komple_types::marketplace::Listing;
        use komple_utils::{query_collection_address, query_token_owner};

        mod fixed_tokens {
            use cosmwasm_std::StdError;
            use komple_utils::{funds::FundsError, query_module_address, query_token_locks};

            use super::*;

            #[test]
            fn test_happy_path_with_marbu() {
                let mut app = mock_app();
                let hub_addr = setup_hub_module(&mut app, true);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, hub_addr.clone());

                // Update public permission settings
                // Creator will be creating the collection
                let msg = MintExecuteMsg::UpdatePublicCollectionCreation {
                    public_collection_creation: true,
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        mint_module_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    CREATOR,
                    token_module_code_id,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                setup_metadata_module(&mut app, collection_addr.clone(), Metadata::Standard);

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);
                mint_token(&mut app, mint_module_addr.clone(), 1, USER);
                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                give_approval_to_module(
                    &mut app,
                    collection_addr.clone(),
                    USER,
                    &marketplace_module_addr,
                );

                setup_marketplace_listing(
                    &mut app,
                    &mint_module_addr,
                    &marketplace_module_addr,
                    1,
                    1,
                    Uint128::new(1_000),
                );

                let locks = query_token_locks(&app.wrap(), &collection_addr, &1).unwrap();
                assert_eq!(locks.transfer_lock, true);
                assert_eq!(locks.send_lock, true);
                assert_eq!(locks.burn_lock, true);

                let msg = MarketplaceExecuteMsg::Buy {
                    listing_type: Listing::Fixed,
                    collection_id: 1,
                    token_id: 1,
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(RANDOM),
                        marketplace_module_addr.clone(),
                        &msg,
                        &vec![coin(1_000, NATIVE_DENOM)],
                    )
                    .unwrap();

                let msg = MarketplaceQueryMsg::FixedListing {
                    collection_id: 1,
                    token_id: 1,
                };
                let res: Result<Empty, StdError> = app
                    .wrap()
                    .query_wasm_smart(marketplace_module_addr.clone(), &msg);
                assert!(res.is_err());

                let locks = query_token_locks(&app.wrap(), &collection_addr, &1).unwrap();
                assert_eq!(locks.transfer_lock, false);
                assert_eq!(locks.send_lock, false);
                assert_eq!(locks.burn_lock, false);

                let owner = query_token_owner(&app.wrap(), &collection_addr, &1).unwrap();
                assert_eq!(owner, Addr::unchecked(RANDOM));

                // Buyer balance
                let balance = app.wrap().query_balance(RANDOM, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(999_000));

                // Owner balance
                let balance = app.wrap().query_balance(USER, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(1_000_920));

                // Komple fee
                let balance = app.wrap().query_balance("contract0", NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(40));

                // Community fee
                let balance = app
                    .wrap()
                    .query_balance("juno..community", NATIVE_DENOM)
                    .unwrap();
                assert_eq!(balance.amount, Uint128::new(20));

                // Marketplace owner fee
                let balance = app.wrap().query_balance(ADMIN, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(20));

                let fee_module_code_id = app.store_code(fee_module());
                let msg = HubExecuteMsg::RegisterModule {
                    module: Modules::Fee.to_string(),
                    msg: to_binary(&FeeModuleInstantiateMsg {
                        admin: ADMIN.to_string(),
                    })
                    .unwrap(),
                    code_id: fee_module_code_id,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &[])
                    .unwrap();
                let fee_module_addr =
                    query_module_address(&app.wrap(), &hub_addr, Modules::Fee).unwrap();

                // Setup admin royalty for 10 percent
                set_royalties(&mut app, &fee_module_addr, &1, "0.1");

                setup_marketplace_listing(
                    &mut app,
                    &mint_module_addr,
                    &marketplace_module_addr,
                    1,
                    2,
                    Uint128::new(1_000),
                );

                let msg = MarketplaceExecuteMsg::Buy {
                    listing_type: Listing::Fixed,
                    collection_id: 1,
                    token_id: 2,
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(RANDOM),
                        marketplace_module_addr.clone(),
                        &msg,
                        &vec![coin(1_000, NATIVE_DENOM)],
                    )
                    .unwrap();

                let owner = query_token_owner(&app.wrap(), &collection_addr, &1).unwrap();
                assert_eq!(owner, Addr::unchecked(RANDOM));

                // Buyer balance
                let balance = app.wrap().query_balance(RANDOM, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(998_000));

                // Owner balance
                let balance = app.wrap().query_balance(USER, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(1_001_740));

                // Komple fee
                let balance = app.wrap().query_balance("contract0", NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(80));

                // Community fee
                let balance = app
                    .wrap()
                    .query_balance("juno..community", NATIVE_DENOM)
                    .unwrap();
                assert_eq!(balance.amount, Uint128::new(40));

                // Marketplace owner
                let balance = app.wrap().query_balance(ADMIN, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(40));

                // Creator royalty fee
                let balance = app.wrap().query_balance(CREATOR, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(100));

                // Setup admin royalty for 10 percent
                set_royalties(&mut app, &fee_module_addr, &1, "0.05");

                setup_marketplace_listing(
                    &mut app,
                    &mint_module_addr,
                    &marketplace_module_addr,
                    1,
                    3,
                    Uint128::new(998_000),
                );

                let msg = MarketplaceExecuteMsg::Buy {
                    listing_type: Listing::Fixed,
                    collection_id: 1,
                    token_id: 3,
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(RANDOM),
                        marketplace_module_addr.clone(),
                        &msg,
                        &vec![coin(998_000, NATIVE_DENOM)],
                    )
                    .unwrap();

                // Buyer balance
                let balance = app.wrap().query_balance(RANDOM, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(0));

                // Owner balance
                let balance = app.wrap().query_balance(USER, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(1_870_000));

                // Komple fee
                let balance = app.wrap().query_balance("contract0", NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(40_000));

                // Community fee
                let balance = app
                    .wrap()
                    .query_balance("juno..community", NATIVE_DENOM)
                    .unwrap();
                assert_eq!(balance.amount, Uint128::new(20_000));

                // Marketplace owner
                let balance = app.wrap().query_balance(ADMIN, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(20_000));

                // Creator royalty fee
                let balance = app.wrap().query_balance(CREATOR, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(50_000));
            }

            #[test]
            fn test_happy_path_without_marbu() {
                let mut app = mock_app();
                let hub_addr = setup_hub_module(&mut app, false);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, hub_addr.clone());

                // Register and setup fee module
                let fee_module_code_id = app.store_code(fee_module());
                let msg = HubExecuteMsg::RegisterModule {
                    module: Modules::Fee.to_string(),
                    msg: to_binary(&FeeModuleInstantiateMsg {
                        admin: ADMIN.to_string(),
                    })
                    .unwrap(),
                    code_id: fee_module_code_id,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &[])
                    .unwrap();
                let fee_module_addr =
                    query_module_address(&app.wrap(), &hub_addr, Modules::Fee).unwrap();
                setup_fee_module(&mut app, &fee_module_addr);

                // Update public permission settings
                // Creator will be creating the collection
                let msg = MintExecuteMsg::UpdatePublicCollectionCreation {
                    public_collection_creation: true,
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        mint_module_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    CREATOR,
                    token_module_code_id,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                setup_metadata_module(&mut app, collection_addr.clone(), Metadata::Standard);

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);
                mint_token(&mut app, mint_module_addr.clone(), 1, USER);
                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                give_approval_to_module(
                    &mut app,
                    collection_addr.clone(),
                    USER,
                    &marketplace_module_addr,
                );

                setup_marketplace_listing(
                    &mut app,
                    &mint_module_addr,
                    &marketplace_module_addr,
                    1,
                    1,
                    Uint128::new(1_000),
                );

                let locks = query_token_locks(&app.wrap(), &collection_addr, &1).unwrap();
                assert_eq!(locks.transfer_lock, true);
                assert_eq!(locks.send_lock, true);
                assert_eq!(locks.burn_lock, true);

                let msg = MarketplaceExecuteMsg::Buy {
                    listing_type: Listing::Fixed,
                    collection_id: 1,
                    token_id: 1,
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(RANDOM),
                        marketplace_module_addr.clone(),
                        &msg,
                        &vec![coin(1_000, NATIVE_DENOM)],
                    )
                    .unwrap();

                let msg = MarketplaceQueryMsg::FixedListing {
                    collection_id: 1,
                    token_id: 1,
                };
                let res: Result<Empty, StdError> = app
                    .wrap()
                    .query_wasm_smart(marketplace_module_addr.clone(), &msg);
                assert!(res.is_err());

                let locks = query_token_locks(&app.wrap(), &collection_addr, &1).unwrap();
                assert_eq!(locks.transfer_lock, false);
                assert_eq!(locks.send_lock, false);
                assert_eq!(locks.burn_lock, false);

                let owner = query_token_owner(&app.wrap(), &collection_addr, &1).unwrap();
                assert_eq!(owner, Addr::unchecked(RANDOM));

                // Buyer balance
                let balance = app.wrap().query_balance(RANDOM, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(999_000));

                // Owner balance
                let balance = app.wrap().query_balance(USER, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(1_000_920));

                // Komple fee
                let balance = app.wrap().query_balance("contract0", NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(40));

                // Community fee
                let balance = app
                    .wrap()
                    .query_balance("juno..community", NATIVE_DENOM)
                    .unwrap();
                assert_eq!(balance.amount, Uint128::new(20));

                // Setup admin royalty for 10 percent
                set_royalties(&mut app, &fee_module_addr, &1, "0.1");

                setup_marketplace_listing(
                    &mut app,
                    &mint_module_addr,
                    &marketplace_module_addr,
                    1,
                    2,
                    Uint128::new(1_000),
                );

                let msg = MarketplaceExecuteMsg::Buy {
                    listing_type: Listing::Fixed,
                    collection_id: 1,
                    token_id: 2,
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(RANDOM),
                        marketplace_module_addr.clone(),
                        &msg,
                        &vec![coin(1_000, NATIVE_DENOM)],
                    )
                    .unwrap();

                let owner = query_token_owner(&app.wrap(), &collection_addr, &1).unwrap();
                assert_eq!(owner, Addr::unchecked(RANDOM));

                // Buyer balance
                let balance = app.wrap().query_balance(RANDOM, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(998_000));

                // Owner balance
                let balance = app.wrap().query_balance(USER, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(1_001_740));

                // Komple fee
                let balance = app.wrap().query_balance("contract0", NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(80));

                // Community fee
                let balance = app
                    .wrap()
                    .query_balance("juno..community", NATIVE_DENOM)
                    .unwrap();
                assert_eq!(balance.amount, Uint128::new(40));

                // Creator royalty fee
                let balance = app.wrap().query_balance(CREATOR, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(100));

                set_royalties(&mut app, &fee_module_addr, &1, "0.05");

                setup_marketplace_listing(
                    &mut app,
                    &mint_module_addr,
                    &marketplace_module_addr,
                    1,
                    3,
                    Uint128::new(998_000),
                );

                let msg = MarketplaceExecuteMsg::Buy {
                    listing_type: Listing::Fixed,
                    collection_id: 1,
                    token_id: 3,
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(RANDOM),
                        marketplace_module_addr.clone(),
                        &msg,
                        &vec![coin(998_000, NATIVE_DENOM)],
                    )
                    .unwrap();

                // Buyer balance
                let balance = app.wrap().query_balance(RANDOM, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(0));

                // Owner balance
                let balance = app.wrap().query_balance(USER, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(1_870_000));

                // Komple fee
                let balance = app.wrap().query_balance("contract0", NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(40_000));

                // Community fee
                let balance = app
                    .wrap()
                    .query_balance("juno..community", NATIVE_DENOM)
                    .unwrap();
                assert_eq!(balance.amount, Uint128::new(20_000));

                // Creator royalty fee
                let balance = app.wrap().query_balance(CREATOR, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(50_000));
            }

            #[test]
            fn test_invalid_funds() {
                let mut app = mock_app();

                let hub_addr = setup_hub_module(&mut app, false);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, hub_addr.clone());

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                setup_metadata_module(&mut app, collection_addr.clone(), Metadata::Standard);

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                setup_marketplace_listing(
                    &mut app,
                    &mint_module_addr,
                    &marketplace_module_addr,
                    1,
                    1,
                    Uint128::new(1_000_000),
                );

                let buy_msg = MarketplaceExecuteMsg::Buy {
                    listing_type: Listing::Fixed,
                    collection_id: 1,
                    token_id: 1,
                };

                let err = app
                    .execute_contract(
                        Addr::unchecked(RANDOM),
                        marketplace_module_addr.clone(),
                        &buy_msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    FundsError::MissingFunds {}.to_string()
                );

                let err = app
                    .execute_contract(
                        Addr::unchecked(RANDOM_2),
                        marketplace_module_addr.clone(),
                        &buy_msg,
                        &vec![coin(1_000_000, TEST_DENOM)],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    FundsError::InvalidDenom {
                        got: TEST_DENOM.to_string(),
                        expected: NATIVE_DENOM.to_string()
                    }
                    .to_string()
                );

                let err = app
                    .execute_contract(
                        Addr::unchecked(RANDOM),
                        marketplace_module_addr.clone(),
                        &buy_msg,
                        &vec![coin(100, NATIVE_DENOM)],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    FundsError::InvalidFunds {
                        got: "100".to_string(),
                        expected: "1000000".to_string()
                    }
                    .to_string()
                );
            }

            #[test]
            fn test_self_purchase() {
                let mut app = mock_app();
                let hub_addr = setup_hub_module(&mut app, false);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, hub_addr.clone());

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                setup_metadata_module(&mut app, collection_addr.clone(), Metadata::Standard);

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                setup_marketplace_listing(
                    &mut app,
                    &mint_module_addr,
                    &marketplace_module_addr,
                    1,
                    1,
                    Uint128::new(1_000_000),
                );

                let buy_msg = MarketplaceExecuteMsg::Buy {
                    listing_type: Listing::Fixed,
                    collection_id: 1,
                    token_id: 1,
                };

                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        marketplace_module_addr.clone(),
                        &buy_msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    MarketplaceContractError::SelfPurchase {}.to_string()
                );
            }
        }
    }
}

mod queries {
    use komple_marketplace_module::{msg::QueryMsg, state::FixedListing};

    use super::*;

    #[test]
    fn test_fixed_listings() {
        let mut app = mock_app();
        let hub_addr = setup_hub_module(&mut app, false);

        let (mint_module_addr, marketplace_module_addr) = setup_modules(&mut app, hub_addr.clone());

        let token_module_code_id = app.store_code(token_module());
        create_collection(
            &mut app,
            mint_module_addr.clone(),
            ADMIN,
            token_module_code_id,
        );
        create_collection(
            &mut app,
            mint_module_addr.clone(),
            ADMIN,
            token_module_code_id,
        );

        let collection_addr_1 =
            query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();
        setup_metadata_module(&mut app, collection_addr_1.clone(), MetadataType::Standard);

        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 1, USER);

        setup_marketplace_listing(
            &mut app,
            &mint_module_addr,
            &marketplace_module_addr,
            1,
            1,
            Uint128::new(1_000_000),
        );
        setup_marketplace_listing(
            &mut app,
            &mint_module_addr,
            &marketplace_module_addr,
            1,
            7,
            Uint128::new(1_000_000),
        );
        setup_marketplace_listing(
            &mut app,
            &mint_module_addr,
            &marketplace_module_addr,
            1,
            4,
            Uint128::new(1_000_000),
        );

        let msg = QueryMsg::FixedListings {
            collection_id: 1,
            start_after: None,
            limit: None,
        };
        let res: ResponseWrapper<Vec<FixedListing>> = app
            .wrap()
            .query_wasm_smart(marketplace_module_addr.clone(), &msg)
            .unwrap();
        assert_eq!(res.data.len(), 3);
        assert_eq!(res.data[0].collection_id, 1);
        assert_eq!(res.data[0].token_id, 1);
        assert_eq!(res.data[1].collection_id, 1);
        assert_eq!(res.data[1].token_id, 4);
        assert_eq!(res.data[2].collection_id, 1);
        assert_eq!(res.data[2].token_id, 7);

        let msg = QueryMsg::FixedListings {
            collection_id: 1,
            start_after: Some(4),
            limit: Some(2),
        };
        let res: ResponseWrapper<Vec<FixedListing>> = app
            .wrap()
            .query_wasm_smart(marketplace_module_addr.clone(), &msg)
            .unwrap();
        assert_eq!(res.data.len(), 1);
        assert_eq!(res.data[0].collection_id, 1);
        assert_eq!(res.data[0].token_id, 7);
    }
}
