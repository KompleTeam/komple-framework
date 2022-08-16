use controller_contract::msg::{
    ExecuteMsg as ControllerExecuteMsg, InstantiateMsg as ControllerInstantiateMsg,
    QueryMsg as ControllerQueryMsg,
};
use cosmwasm_std::{Addr, Coin, Decimal, Empty, Timestamp, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_fee_contract::msg::InstantiateMsg as FeeContractInstantiateMsg;
use komple_types::collection::Collections;
use komple_types::metadata::Metadata as MetadataType;
use komple_types::module::Modules;
use komple_types::query::ResponseWrapper;
use komple_utils::query_collection_address;
use marketplace_module::msg::ExecuteMsg;
use metadata_contract::msg::ExecuteMsg as MetadataExecuteMsg;
use metadata_contract::state::{MetaInfo, Trait};
use mint_module::msg::ExecuteMsg as MintExecuteMsg;
use token_contract::{
    msg::{
        ExecuteMsg as TokenExecuteMsg, InstantiateMsg as TokenInstantiateMsg,
        QueryMsg as TokenQueryMsg, TokenInfo,
    },
    state::{CollectionInfo, Contracts},
};

pub const USER: &str = "juno..user";
pub const RANDOM: &str = "juno..random";
pub const ADMIN: &str = "juno..admin";
pub const RANDOM_2: &str = "juno..random2";
pub const NATIVE_DENOM: &str = "denom";
pub const TEST_DENOM: &str = "test_denom";

pub fn controller_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        controller_contract::contract::execute,
        controller_contract::contract::instantiate,
        controller_contract::contract::query,
    )
    .with_reply(controller_contract::contract::reply);
    Box::new(contract)
}

pub fn mint_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        mint_module::contract::execute,
        mint_module::contract::instantiate,
        mint_module::contract::query,
    )
    .with_reply(mint_module::contract::reply);
    Box::new(contract)
}

pub fn token_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        token_contract::contract::execute,
        token_contract::contract::instantiate,
        token_contract::contract::query,
    )
    .with_reply(token_contract::contract::reply);
    Box::new(contract)
}

pub fn marketplace_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        marketplace_module::contract::execute,
        marketplace_module::contract::instantiate,
        marketplace_module::contract::query,
    );
    Box::new(contract)
}

pub fn metadata_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        metadata_contract::contract::execute,
        metadata_contract::contract::instantiate,
        metadata_contract::contract::query,
    );
    Box::new(contract)
}

pub fn fee_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_fee_contract::contract::execute,
        komple_fee_contract::contract::instantiate,
        komple_fee_contract::contract::query,
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

fn setup_fee_contract(app: &mut App) -> Addr {
    let fee_code_id = app.store_code(fee_contract());

    let msg = FeeContractInstantiateMsg {
        komple_address: ADMIN.to_string(),
        payment_address: "juno..community".to_string(),
    };
    let fee_contract_addr = app
        .instantiate_contract(
            fee_code_id,
            Addr::unchecked(ADMIN),
            &msg,
            &vec![],
            "test",
            None,
        )
        .unwrap();

    fee_contract_addr
}

fn setup_controller_contract(app: &mut App) -> Addr {
    let controller_code_id = app.store_code(controller_contract());

    let msg = ControllerInstantiateMsg {
        name: "Test Controller".to_string(),
        description: "Test Controller".to_string(),
        image: "https://example.com/image.png".to_string(),
        external_link: None,
    };
    let controller_addr = app
        .instantiate_contract(
            controller_code_id,
            Addr::unchecked(ADMIN),
            &msg,
            &vec![],
            "test",
            None,
        )
        .unwrap();

    controller_addr
}

fn setup_modules(app: &mut App, controller_addr: Addr) -> (Addr, Addr) {
    let mint_code_id = app.store_code(mint_module());
    let marketplace_code_id = app.store_code(marketplace_module());

    let msg = ControllerExecuteMsg::InitMintModule {
        code_id: mint_code_id,
    };
    let _ = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            controller_addr.clone(),
            &msg,
            &vec![],
        )
        .unwrap();
    let msg = ControllerExecuteMsg::InitMarketplaceModule {
        code_id: marketplace_code_id,
        native_denom: NATIVE_DENOM.to_string(),
    };
    let _ = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            controller_addr.clone(),
            &msg,
            &vec![],
        )
        .unwrap();

    let msg = ControllerQueryMsg::ModuleAddress(Modules::MintModule);
    let mint_res: ResponseWrapper<Addr> = app
        .wrap()
        .query_wasm_smart(controller_addr.clone(), &msg)
        .unwrap();
    let msg = ControllerQueryMsg::ModuleAddress(Modules::MarketplaceModule);
    let marketplace_res: ResponseWrapper<Addr> = app
        .wrap()
        .query_wasm_smart(controller_addr.clone(), &msg)
        .unwrap();

    (mint_res.data, marketplace_res.data)
}

pub fn create_collection(
    app: &mut App,
    mint_module_addr: Addr,
    token_contract_code_id: u64,
    per_address_limit: Option<u32>,
    start_time: Option<Timestamp>,
    collection_type: Collections,
    linked_collections: Option<Vec<u32>>,
    unit_price: Option<Coin>,
    max_token_limit: Option<u32>,
    royalty_share: Option<Decimal>,
) {
    let collection_info = CollectionInfo {
        collection_type,
        name: "Test Collection".to_string(),
        description: "Test Collection".to_string(),
        image: "https://image.com".to_string(),
        external_link: None,
    };
    let token_info = TokenInfo {
        symbol: "TEST".to_string(),
        minter: mint_module_addr.to_string(),
    };
    let msg = MintExecuteMsg::CreateCollection {
        code_id: token_contract_code_id,
        token_instantiate_msg: TokenInstantiateMsg {
            admin: ADMIN.to_string(),
            collection_info,
            token_info,
            per_address_limit,
            start_time,
            unit_price,
            native_denom: NATIVE_DENOM.to_string(),
            max_token_limit,
            royalty_share,
        },
        linked_collections,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), mint_module_addr, &msg, &vec![])
        .unwrap();
}

pub fn setup_metadata_contract(
    app: &mut App,
    token_contract_addr: Addr,
    metadata_type: MetadataType,
) -> Addr {
    let metadata_code_id = app.store_code(metadata_contract());

    let msg = TokenExecuteMsg::InitMetadataContract {
        code_id: metadata_code_id,
        metadata_type,
    };
    let _ = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            token_contract_addr.clone(),
            &msg,
            &[],
        )
        .unwrap();

    let res: ResponseWrapper<Contracts> = app
        .wrap()
        .query_wasm_smart(token_contract_addr.clone(), &TokenQueryMsg::Contracts {})
        .unwrap();
    res.data.metadata.unwrap()
}

pub fn setup_metadata(app: &mut App, metadata_contract_addr: Addr) {
    let meta_info = MetaInfo {
        image: Some("https://some-image.com".to_string()),
        external_url: None,
        description: Some("Some description".to_string()),
        youtube_url: None,
        animation_url: None,
    };
    let attributes = vec![
        Trait {
            trait_type: "trait_1".to_string(),
            value: "value_1".to_string(),
        },
        Trait {
            trait_type: "trait_2".to_string(),
            value: "value_2".to_string(),
        },
    ];
    let msg = MetadataExecuteMsg::AddMetadata {
        meta_info,
        attributes,
    };
    let _ = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            metadata_contract_addr.clone(),
            &msg,
            &vec![],
        )
        .unwrap();
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

pub fn setup_token_contract_operators(
    app: &mut App,
    token_contract_addr: Addr,
    addrs: Vec<String>,
) {
    let msg = TokenExecuteMsg::UpdateOperators { addrs };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), token_contract_addr, &msg, &vec![])
        .unwrap();
}

pub fn give_approval_to_module(
    app: &mut App,
    token_contract_addr: Addr,
    owner: &str,
    operator_addr: &Addr,
) {
    let msg = TokenExecuteMsg::ApproveAll {
        operator: operator_addr.to_string(),
        expires: None,
    };
    let _ = app
        .execute_contract(Addr::unchecked(owner), token_contract_addr, &msg, &vec![])
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

    setup_token_contract_operators(
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

    use controller_contract::ContractError;
    use komple_utils::query_module_address;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        setup_fee_contract(&mut app);
        let controller_addr = setup_controller_contract(&mut app);
        let marketplace_module_code_id = app.store_code(marketplace_module());

        let msg = ControllerExecuteMsg::InitMarketplaceModule {
            code_id: marketplace_module_code_id,
            native_denom: "test".to_string(),
        };
        let _ = app.execute_contract(
            Addr::unchecked(ADMIN),
            controller_addr.clone(),
            &msg,
            &vec![],
        );

        let res = query_module_address(&app.wrap(), &controller_addr, Modules::MarketplaceModule)
            .unwrap();
        assert_eq!(res, "contract2")
    }

    #[test]
    fn test_invalid_sender() {
        let mut app = mock_app();
        let controller_addr = setup_controller_contract(&mut app);
        let marketplace_module_code_id = app.store_code(marketplace_module());

        let msg = ControllerExecuteMsg::InitMarketplaceModule {
            code_id: marketplace_module_code_id,
            native_denom: "test".to_string(),
        };
        let err = app
            .execute_contract(
                Addr::unchecked(USER),
                controller_addr.clone(),
                &msg,
                &vec![],
            )
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
    use komple_types::collection::Collections;
    use marketplace_module::{
        msg::{ExecuteMsg as MarketplaceExecuteMsg, QueryMsg as MarketplaceQueryMsg},
        ContractError as MarketplaceContractError,
    };
    use token_contract::msg::ExecuteMsg as TokenExecuteMsg;
    use token_contract::ContractError as TokenContractError;

    use komple_types::metadata::Metadata;

    mod listing {
        use super::*;

        mod fixed_tokens {
            use super::*;

            use komple_types::{metadata::Metadata, query::ResponseWrapper, tokens::Locks};
            use komple_utils::{query_collection_address, query_token_locks};
            use marketplace_module::state::FixedListing;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                setup_fee_contract(&mut app);
                let controller_addr = setup_controller_contract(&mut app);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, controller_addr.clone());

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                    None,
                    None,
                    None,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                let metadata_contract_addr =
                    setup_metadata_contract(&mut app, collection_addr.clone(), Metadata::OneToOne);
                setup_metadata(&mut app, metadata_contract_addr.clone());

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                setup_token_contract_operators(
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
                setup_fee_contract(&mut app);
                let controller_addr = setup_controller_contract(&mut app);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, controller_addr.clone());

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                    None,
                    None,
                    None,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                let metadata_contract_addr =
                    setup_metadata_contract(&mut app, collection_addr.clone(), Metadata::OneToOne);
                setup_metadata(&mut app, metadata_contract_addr.clone());

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
                setup_fee_contract(&mut app);
                let controller_addr = setup_controller_contract(&mut app);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, controller_addr.clone());

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                    None,
                    None,
                    None,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                let metadata_contract_addr =
                    setup_metadata_contract(&mut app, collection_addr.clone(), Metadata::OneToOne);
                setup_metadata(&mut app, metadata_contract_addr.clone());

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
                let msg = TokenExecuteMsg::UpdateTokenLock {
                    token_id: "1".to_string(),
                    locks: transfer_lock.clone(),
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

                let msg = TokenExecuteMsg::UpdateTokenLock {
                    token_id: "1".to_string(),
                    locks: unlock.clone(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        collection_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = TokenExecuteMsg::UpdateLocks {
                    locks: transfer_lock.clone(),
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

                let msg = TokenExecuteMsg::UpdateLocks {
                    locks: unlock.clone(),
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
                setup_fee_contract(&mut app);
                let controller_addr = setup_controller_contract(&mut app);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, controller_addr.clone());

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                    None,
                    None,
                    None,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                let metadata_contract_addr =
                    setup_metadata_contract(&mut app, collection_addr.clone(), Metadata::OneToOne);
                setup_metadata(&mut app, metadata_contract_addr.clone());

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
                setup_fee_contract(&mut app);
                let controller_addr = setup_controller_contract(&mut app);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, controller_addr.clone());

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                    None,
                    None,
                    None,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                let metadata_contract_addr =
                    setup_metadata_contract(&mut app, collection_addr.clone(), Metadata::OneToOne);
                setup_metadata(&mut app, metadata_contract_addr.clone());

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                setup_token_contract_operators(
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
                setup_fee_contract(&mut app);
                let controller_addr = setup_controller_contract(&mut app);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, controller_addr.clone());

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                    None,
                    None,
                    None,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                let metadata_contract_addr =
                    setup_metadata_contract(&mut app, collection_addr.clone(), Metadata::OneToOne);
                setup_metadata(&mut app, metadata_contract_addr.clone());

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                setup_token_contract_operators(
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
                setup_fee_contract(&mut app);
                let controller_addr = setup_controller_contract(&mut app);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, controller_addr.clone());

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                    None,
                    None,
                    None,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                let metadata_contract_addr =
                    setup_metadata_contract(&mut app, collection_addr.clone(), Metadata::OneToOne);
                setup_metadata(&mut app, metadata_contract_addr.clone());

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                setup_token_contract_operators(
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

                setup_token_contract_operators(&mut app, collection_addr.clone(), vec![]);

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
        use komple_types::{marketplace::Listing, query::ResponseWrapper};
        use marketplace_module::state::FixedListing;

        use super::*;

        mod fixed_tokens {
            use komple_utils::query_collection_address;

            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                setup_fee_contract(&mut app);
                let controller_addr = setup_controller_contract(&mut app);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, controller_addr.clone());

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                    None,
                    None,
                    None,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                let metadata_contract_addr =
                    setup_metadata_contract(&mut app, collection_addr.clone(), Metadata::OneToOne);
                setup_metadata(&mut app, metadata_contract_addr.clone());

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
                setup_fee_contract(&mut app);
                let controller_addr = setup_controller_contract(&mut app);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, controller_addr.clone());

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                    None,
                    None,
                    None,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                let metadata_contract_addr =
                    setup_metadata_contract(&mut app, collection_addr.clone(), Metadata::OneToOne);
                setup_metadata(&mut app, metadata_contract_addr.clone());

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
            use std::str::FromStr;

            use cosmwasm_std::{Decimal, StdError};
            use komple_utils::{query_token_locks, FundsError};

            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                setup_fee_contract(&mut app);
                let controller_addr = setup_controller_contract(&mut app);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, controller_addr.clone());

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                    None,
                    None,
                    None,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                let metadata_contract_addr =
                    setup_metadata_contract(&mut app, collection_addr.clone(), Metadata::OneToOne);
                setup_metadata(&mut app, metadata_contract_addr.clone());
                setup_metadata(&mut app, metadata_contract_addr.clone());
                setup_metadata(&mut app, metadata_contract_addr.clone());

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
                assert_eq!(balance.amount, Uint128::new(1_000_930));

                // Marketplace fee
                let balance = app.wrap().query_balance("contract0", NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(70));

                // Admin royalty fee
                let balance = app.wrap().query_balance(ADMIN, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(0));

                // Setup admin royalty for 10 percent
                let msg = TokenExecuteMsg::UpdateRoyaltyShare {
                    royalty_share: Some(Decimal::from_str("0.1").unwrap()),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        collection_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

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
                assert_eq!(balance.amount, Uint128::new(1_001_760));

                // Marketplace fee
                let balance = app.wrap().query_balance("contract0", NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(140));

                // Admin royalty fee
                let balance = app.wrap().query_balance(ADMIN, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(100));

                let msg = TokenExecuteMsg::UpdateRoyaltyShare {
                    royalty_share: Some(Decimal::from_str("0.05").unwrap()),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        collection_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

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
                assert_eq!(balance.amount, Uint128::new(1_880_000));

                // Marketplace fee
                let balance = app.wrap().query_balance("contract0", NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(70_000));

                // Admin royalty fee
                let balance = app.wrap().query_balance(ADMIN, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(50_000));
            }

            #[test]
            fn test_invalid_funds() {
                let mut app = mock_app();
                setup_fee_contract(&mut app);
                let controller_addr = setup_controller_contract(&mut app);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, controller_addr.clone());

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                    None,
                    None,
                    None,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                let metadata_contract_addr =
                    setup_metadata_contract(&mut app, collection_addr.clone(), Metadata::OneToOne);
                setup_metadata(&mut app, metadata_contract_addr.clone());

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
                setup_fee_contract(&mut app);
                let controller_addr = setup_controller_contract(&mut app);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, controller_addr.clone());

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                    None,
                    None,
                    None,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                let metadata_contract_addr =
                    setup_metadata_contract(&mut app, collection_addr.clone(), Metadata::OneToOne);
                setup_metadata(&mut app, metadata_contract_addr.clone());

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
    use marketplace_module::{msg::QueryMsg, state::FixedListing};

    use super::*;

    #[test]
    fn test_fixed_listings() {
        let mut app = mock_app();
        setup_fee_contract(&mut app);
        let controller_addr = setup_controller_contract(&mut app);

        let (mint_module_addr, marketplace_module_addr) =
            setup_modules(&mut app, controller_addr.clone());

        let token_contract_code_id = app.store_code(token_contract());
        create_collection(
            &mut app,
            mint_module_addr.clone(),
            token_contract_code_id,
            None,
            None,
            Collections::Normal,
            None,
            None,
            None,
            None,
        );
        create_collection(
            &mut app,
            mint_module_addr.clone(),
            token_contract_code_id,
            None,
            None,
            Collections::Normal,
            None,
            None,
            None,
            None,
        );

        let collection_addr_1 =
            query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();
        let metadata_contract_addr_1 =
            setup_metadata_contract(&mut app, collection_addr_1.clone(), MetadataType::OneToOne);
        setup_metadata(&mut app, metadata_contract_addr_1.clone());
        setup_metadata(&mut app, metadata_contract_addr_1.clone());
        setup_metadata(&mut app, metadata_contract_addr_1.clone());
        setup_metadata(&mut app, metadata_contract_addr_1.clone());
        setup_metadata(&mut app, metadata_contract_addr_1.clone());
        setup_metadata(&mut app, metadata_contract_addr_1.clone());
        setup_metadata(&mut app, metadata_contract_addr_1.clone());
        setup_metadata(&mut app, metadata_contract_addr_1.clone());
        setup_metadata(&mut app, metadata_contract_addr_1.clone());

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
