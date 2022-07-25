use controller_contract::msg::ExecuteMsg;
use cosmwasm_std::Addr;
use cw_multi_test::Executor;

pub mod helpers;
use helpers::{
    create_collection, get_modules_addresses, give_approval_to_module, marketplace_module,
    mint_token, mock_app, proper_instantiate, setup_all_modules, setup_marketplace_listing,
    setup_royalty_contract, setup_token_contract_operators, token_contract, ADMIN, NATIVE_DENOM,
    RANDOM, RANDOM_2, TEST_DENOM, USER,
};

mod initialization {
    use super::*;

    use rift_types::module::Modules;

    use controller_contract::ContractError;
    use rift_utils::query_module_address;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let controller_addr = proper_instantiate(&mut app);
        let marketplace_module_code_id = app.store_code(marketplace_module());

        let msg = ExecuteMsg::InitMarketplaceModule {
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
        assert_eq!(res, "contract1")
    }

    #[test]
    fn test_invalid_sender() {
        let mut app = mock_app();
        let controller_addr = proper_instantiate(&mut app);
        let marketplace_module_code_id = app.store_code(marketplace_module());

        let msg = ExecuteMsg::InitMarketplaceModule {
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
    use marketplace_module::{
        msg::{ExecuteMsg as MarketplaceExecuteMsg, QueryMsg as MarketplaceQueryMsg},
        ContractError as MarketplaceContractError,
    };
    use rift_types::collection::Collections;
    use token_contract::msg::ExecuteMsg as TokenExecuteMsg;
    use token_contract::ContractError as TokenContractError;

    mod listing {
        use super::*;

        mod fixed_tokens {
            use super::*;

            use marketplace_module::state::FixedListing;
            use rift_types::{query::ResponseWrapper, tokens::Locks};
            use rift_utils::{query_collection_address, query_token_operation_lock};

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let controller_addr = proper_instantiate(&mut app);

                setup_all_modules(&mut app, controller_addr.clone());

                let (mint_module_addr, _, _, marketplace_module_addr) =
                    get_modules_addresses(&mut app, &controller_addr);

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                );

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                setup_token_contract_operators(
                    &mut app,
                    collection_addr.clone(),
                    vec![marketplace_module_addr.to_string()],
                );

                let lock = query_token_operation_lock(&app.wrap(), &collection_addr).unwrap();
                assert_eq!(lock, false);

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

                let lock = query_token_operation_lock(&app.wrap(), &collection_addr).unwrap();
                assert_eq!(lock, true);
            }

            #[test]
            fn test_invalid_owner() {
                let mut app = mock_app();
                let controller_addr = proper_instantiate(&mut app);

                setup_all_modules(&mut app, controller_addr.clone());

                let (mint_module_addr, _, _, marketplace_module_addr) =
                    get_modules_addresses(&mut app, &controller_addr);

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                );

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
                let controller_addr = proper_instantiate(&mut app);

                setup_all_modules(&mut app, controller_addr.clone());

                let (mint_module_addr, _, _, marketplace_module_addr) =
                    get_modules_addresses(&mut app, &controller_addr);

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                );

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

                let msg = TokenExecuteMsg::UpdateOperationLock { lock: true };
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
            }

            #[test]
            fn test_invalid_operator() {
                let mut app = mock_app();
                let controller_addr = proper_instantiate(&mut app);

                setup_all_modules(&mut app, controller_addr.clone());

                let (mint_module_addr, _, _, marketplace_module_addr) =
                    get_modules_addresses(&mut app, &controller_addr);

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                );

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
        use rift_utils::{query_collection_address, query_token_operation_lock};

        mod fixed_tokens {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let controller_addr = proper_instantiate(&mut app);

                setup_all_modules(&mut app, controller_addr.clone());

                let (mint_module_addr, _, _, marketplace_module_addr) =
                    get_modules_addresses(&mut app, &controller_addr);

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                );

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

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

                let lock = query_token_operation_lock(&app.wrap(), &collection_addr).unwrap();
                assert_eq!(lock, true);

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

                let msg = MarketplaceQueryMsg::FixedListing {
                    collection_id: 1,
                    token_id: 1,
                };
                let res: Result<Empty, cosmwasm_std::StdError> =
                    app.wrap().query_wasm_smart(marketplace_module_addr, &msg);
                assert!(res.is_err());

                let lock = query_token_operation_lock(&app.wrap(), &collection_addr).unwrap();
                assert_eq!(lock, false);
            }

            #[test]
            fn test_invalid_owner() {
                let mut app = mock_app();
                let controller_addr = proper_instantiate(&mut app);

                setup_all_modules(&mut app, controller_addr.clone());

                let (mint_module_addr, _, _, marketplace_module_addr) =
                    get_modules_addresses(&mut app, &controller_addr);

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                );

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

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
                let controller_addr = proper_instantiate(&mut app);

                setup_all_modules(&mut app, controller_addr.clone());

                let (mint_module_addr, _, _, marketplace_module_addr) =
                    get_modules_addresses(&mut app, &controller_addr);

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                );

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

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
        use marketplace_module::state::FixedListing;
        use rift_types::{marketplace::Listing, query::ResponseWrapper};

        use super::*;

        mod fixed_tokens {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let controller_addr = proper_instantiate(&mut app);

                setup_all_modules(&mut app, controller_addr.clone());

                let (mint_module_addr, _, _, marketplace_module_addr) =
                    get_modules_addresses(&mut app, &controller_addr);

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                );

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                setup_marketplace_listing(
                    &mut app,
                    &controller_addr,
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
                let controller_addr = proper_instantiate(&mut app);

                setup_all_modules(&mut app, controller_addr.clone());

                let (mint_module_addr, _, _, marketplace_module_addr) =
                    get_modules_addresses(&mut app, &controller_addr);

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                );

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                setup_marketplace_listing(
                    &mut app,
                    &controller_addr,
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
        use rift_types::marketplace::Listing;
        use rift_utils::{query_collection_address, query_token_owner};

        mod fixed_tokens {
            use std::str::FromStr;

            use cosmwasm_std::Decimal;
            use rift_types::royalty::Royalty;

            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let controller_addr = proper_instantiate(&mut app);

                setup_all_modules(&mut app, controller_addr.clone());

                let (mint_module_addr, _, _, marketplace_module_addr) =
                    get_modules_addresses(&mut app, &controller_addr);

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                );

                let collection_addr =
                    query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);
                mint_token(&mut app, mint_module_addr.clone(), 1, USER);
                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                give_approval_to_module(
                    &mut app,
                    collection_addr.clone(),
                    USER,
                    &marketplace_module_addr,
                );

                setup_marketplace_listing(&mut app, &controller_addr, 1, 1, Uint128::new(1_000));

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

                let owner = query_token_owner(&app.wrap(), &collection_addr, &1).unwrap();
                assert_eq!(owner, Addr::unchecked(RANDOM));

                // Buyer balance
                let balance = app.wrap().query_balance(RANDOM, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(999_000));

                // Owner balance
                let balance = app.wrap().query_balance(USER, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(1_000_950));

                // Marketplace fee
                let balance = app.wrap().query_balance("juno..xxx", NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(50));

                // Admin royalty fee
                let balance = app.wrap().query_balance(ADMIN, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(0));

                // Setup admin royalty for 10 percent
                setup_royalty_contract(
                    &mut app,
                    collection_addr.clone(),
                    Decimal::from_str("0.1").unwrap(),
                    Royalty::Admin,
                );

                setup_marketplace_listing(&mut app, &controller_addr, 1, 2, Uint128::new(1_000));

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
                assert_eq!(balance.amount, Uint128::new(1_001_800));

                // Marketplace fee
                let balance = app.wrap().query_balance("juno..xxx", NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(100));

                // Admin royalty fee
                let balance = app.wrap().query_balance(ADMIN, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(100));

                setup_royalty_contract(
                    &mut app,
                    collection_addr.clone(),
                    Decimal::from_str("0.05").unwrap(),
                    Royalty::Owners,
                );

                setup_marketplace_listing(&mut app, &controller_addr, 1, 3, Uint128::new(998_000));

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
                assert_eq!(balance.amount, Uint128::new(1_949_900));

                // Marketplace fee
                let balance = app.wrap().query_balance("juno..xxx", NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(50_000));

                // Admin royalty fee
                let balance = app.wrap().query_balance(ADMIN, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(100));
            }

            #[test]
            fn test_invalid_funds() {
                let mut app = mock_app();
                let controller_addr = proper_instantiate(&mut app);

                setup_all_modules(&mut app, controller_addr.clone());

                let (mint_module_addr, _, _, marketplace_module_addr) =
                    get_modules_addresses(&mut app, &controller_addr);

                let token_contract_code_id = app.store_code(token_contract());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    token_contract_code_id,
                    None,
                    None,
                    Collections::Normal,
                    None,
                );

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                setup_marketplace_listing(
                    &mut app,
                    &controller_addr,
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
                    MarketplaceContractError::InvalidFunds {}.to_string()
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
                    MarketplaceContractError::InvalidDenom {}.to_string()
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
                    MarketplaceContractError::InvalidFunds {}.to_string()
                );
            }
        }
    }
}
