use cosmwasm_std::{Addr, Empty, to_binary, Uint128};
use cw721_base::msg::ExecuteMsg as Cw721ExecuteMsg;
use cw_multi_test::Executor;
use komple_hub_module::msg::ExecuteMsg as HubExecuteMsg;
use komple_marketplace_module::msg::{InstantiateMsg, MarketplaceFundInfo};
use komple_marketplace_module::ContractError;
use komple_mint_module::msg::ExecuteMsg as MintExecuteMsg;
use komple_types::module::Modules;
use komple_types::query::ResponseWrapper;
use komple_types::shared::RegisterMsg;
use komple_utils::funds::FundsError;
use komple_utils::storage::StorageHelper;

pub mod helpers;
use helpers::*;

mod initialization {
    use super::*;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let hub_addr = setup_hub_module(&mut app, false);
        let marketplace_module_code_id = app.store_code(marketplace_module());

        let instantiate_msg = Some(
            to_binary(&InstantiateMsg {
                fund_info: MarketplaceFundInfo {
                    is_native: true,
                    denom: NATIVE_DENOM.to_string(),
                    cw20_address: None,
                },
            })
            .unwrap(),
        );
        let msg = HubExecuteMsg::RegisterModule {
            module: Modules::Marketplace.to_string(),
            msg: instantiate_msg,
            code_id: marketplace_module_code_id,
        };
        let _ = app.execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &[]);

        let res = StorageHelper::query_module_address(
            &app.wrap(),
            &hub_addr,
            Modules::Marketplace.to_string(),
        )
        .unwrap();
        assert_eq!(res, "contract1")
    }

    #[test]
    fn test_happy_path_with_fee_module() {
        let mut app = mock_app();
        let hub_addr = setup_hub_module(&mut app, false);
        let marketplace_module_code_id = app.store_code(marketplace_module());
        let fee_module_code_id = app.store_code(fee_module());

        let instantiate_msg = to_binary(&RegisterMsg {
            admin: ADMIN.to_string(),
            data: None,
        })
        .unwrap();
        let msg = HubExecuteMsg::RegisterModule {
            module: Modules::Fee.to_string(),
            msg: Some(instantiate_msg),
            code_id: fee_module_code_id,
        };
        let _ = app.execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &[]);

        let instantiate_msg = Some(
            to_binary(&InstantiateMsg {
                fund_info: MarketplaceFundInfo {
                    is_native: true,
                    denom: NATIVE_DENOM.to_string(),
                    cw20_address: None,
                },
            })
            .unwrap(),
        );
        let msg = HubExecuteMsg::RegisterModule {
            module: Modules::Marketplace.to_string(),
            msg: instantiate_msg,
            code_id: marketplace_module_code_id,
        };
        let _ = app.execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &[]);

        let res = StorageHelper::query_module_address(
            &app.wrap(),
            &hub_addr,
            Modules::Marketplace.to_string(),
        )
        .unwrap();
        assert_eq!(res, "contract2")
    }

    #[test]
    fn test_invalid_sender() {
        let mut app = mock_app();
        let hub_addr = setup_hub_module(&mut app, false);
        let marketplace_module_code_id = app.store_code(marketplace_module());

        let instantiate_msg = to_binary(&RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(
                to_binary(&InstantiateMsg {
                    fund_info: MarketplaceFundInfo {
                        is_native: true,
                        denom: NATIVE_DENOM.to_string(),
                        cw20_address: None,
                    },
                })
                .unwrap(),
            ),
        })
        .unwrap();
        let msg = HubExecuteMsg::RegisterModule {
            module: Modules::Marketplace.to_string(),
            msg: Some(instantiate_msg),
            code_id: marketplace_module_code_id,
        };
        let err = app
            .execute_contract(Addr::unchecked(USER), hub_addr, &msg, &[])
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
        ContractError as MarketplaceContractError,
        msg::{ExecuteMsg as MarketplaceExecuteMsg, QueryMsg as MarketplaceQueryMsg},
    };
    use komple_token_module::msg::ExecuteMsg as TokenExecuteMsg;
    use komple_token_module::ContractError as TokenContractError;

    mod listing {
        use super::*;

        mod fixed_tokens {
            use super::*;

            use komple_marketplace_module::state::FixedListing;
            use komple_types::query::ResponseWrapper;
            use komple_types::modules::token::Locks;
            use komple_utils::storage::StorageHelper;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let hub_addr = setup_hub_module(&mut app, false);

                let (mint_module_addr, marketplace_module_addr) = setup_modules(&mut app, hub_addr);

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

                let collection_addr =
                    StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &1)
                        .unwrap();

                mint_token(&mut app, mint_module_addr, 1, USER);

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
                        &[],
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

                let locks =
                    StorageHelper::query_token_locks(&app.wrap(), &collection_addr, &1).unwrap();
                assert_eq!(locks.transfer_lock, true);
                assert_eq!(locks.send_lock, true);
                assert_eq!(locks.burn_lock, true);
            }

            #[test]
            fn test_invalid_owner() {
                let mut app = mock_app();
                let hub_addr = setup_hub_module(&mut app, false);

                let (mint_module_addr, marketplace_module_addr) = setup_modules(&mut app, hub_addr);

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

                mint_token(&mut app, mint_module_addr, 1, USER);

                let msg = MarketplaceExecuteMsg::ListFixedToken {
                    collection_id: 1,
                    token_id: 1,
                    price: Uint128::new(1_000_000),
                };
                let err = app
                    .execute_contract(Addr::unchecked(RANDOM), marketplace_module_addr, &msg, &[])
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

                let (mint_module_addr, marketplace_module_addr) = setup_modules(&mut app, hub_addr);

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                let listing_msg = MarketplaceExecuteMsg::ListFixedToken {
                    collection_id: 1,
                    token_id: 1,
                    price: Uint128::new(1_000_000),
                };

                let collection_addr =
                    StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &1)
                        .unwrap();

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
                    msg: TokenExecuteMsg::UpdateTokenLocks {
                        token_id: "1".to_string(),
                        locks: transfer_lock.clone(),
                    },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), collection_addr.clone(), &msg, &[])
                    .unwrap();

                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        marketplace_module_addr.clone(),
                        &listing_msg,
                        &[],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    TokenContractError::TransferLocked {}.to_string()
                );

                let msg: Cw721ExecuteMsg<Empty, TokenExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: TokenExecuteMsg::UpdateTokenLocks {
                        token_id: "1".to_string(),
                        locks: unlock.clone(),
                    },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), collection_addr.clone(), &msg, &[])
                    .unwrap();

                let msg: Cw721ExecuteMsg<Empty, TokenExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: TokenExecuteMsg::UpdateLocks {
                        locks: transfer_lock,
                    },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), collection_addr.clone(), &msg, &[])
                    .unwrap();

                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        marketplace_module_addr,
                        &listing_msg,
                        &[],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    TokenContractError::TransferLocked {}.to_string()
                );

                let msg: Cw721ExecuteMsg<Empty, TokenExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: TokenExecuteMsg::UpdateLocks { locks: unlock },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), collection_addr, &msg, &[])
                    .unwrap();
            }

            #[test]
            fn test_invalid_operator() {
                let mut app = mock_app();
                let hub_addr = setup_hub_module(&mut app, false);

                let (mint_module_addr, marketplace_module_addr) = setup_modules(&mut app, hub_addr);

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

                mint_token(&mut app, mint_module_addr, 1, USER);

                let msg = MarketplaceExecuteMsg::ListFixedToken {
                    collection_id: 1,
                    token_id: 1,
                    price: Uint128::new(1_000_000),
                };
                let err = app
                    .execute_contract(Addr::unchecked(USER), marketplace_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().source().unwrap().to_string(),
                    TokenContractError::Unauthorized {}.to_string()
                );
            }

            #[test]
            fn test_existing_listing() {
                let mut app = mock_app();
                let hub_addr = setup_hub_module(&mut app, false);

                let (mint_module_addr, marketplace_module_addr) = setup_modules(&mut app, hub_addr);

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

                let collection_addr =
                    StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &1)
                        .unwrap();

                setup_token_module_operators(
                    &mut app,
                    collection_addr.clone(),
                    vec![marketplace_module_addr.to_string()],
                );

                mint_token(&mut app, mint_module_addr, 1, USER);

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
                        &[],
                    )
                    .unwrap();

                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        marketplace_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::AlreadyListed {}.to_string()
                )
            }
        }
    }

    mod delisting {
        use super::*;

        mod fixed_tokens {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let hub_addr = setup_hub_module(&mut app, false);

                let (mint_module_addr, marketplace_module_addr) = setup_modules(&mut app, hub_addr);

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

                let collection_addr =
                    StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &1)
                        .unwrap();

                mint_token(&mut app, mint_module_addr, 1, USER);

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
                        &[],
                    )
                    .unwrap();

                let locks =
                    StorageHelper::query_token_locks(&app.wrap(), &collection_addr, &1).unwrap();
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
                        &[],
                    )
                    .unwrap();

                let locks =
                    StorageHelper::query_token_locks(&app.wrap(), &collection_addr, &1).unwrap();
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

                let (mint_module_addr, marketplace_module_addr) = setup_modules(&mut app, hub_addr);

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

                let collection_addr =
                    StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &1)
                        .unwrap();

                mint_token(&mut app, mint_module_addr, 1, USER);

                setup_token_module_operators(
                    &mut app,
                    collection_addr,
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
                        &[],
                    )
                    .unwrap();

                let msg = MarketplaceExecuteMsg::DelistFixedToken {
                    collection_id: 1,
                    token_id: 1,
                };
                let err = app
                    .execute_contract(Addr::unchecked(RANDOM), marketplace_module_addr, &msg, &[])
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

                let (mint_module_addr, marketplace_module_addr) = setup_modules(&mut app, hub_addr);

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

                let collection_addr =
                    StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &1)
                        .unwrap();

                mint_token(&mut app, mint_module_addr, 1, USER);

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
                        &[],
                    )
                    .unwrap();

                setup_token_module_operators(&mut app, collection_addr, vec![]);

                let msg = MarketplaceExecuteMsg::DelistFixedToken {
                    collection_id: 1,
                    token_id: 1,
                };
                let err = app
                    .execute_contract(Addr::unchecked(USER), marketplace_module_addr, &msg, &[])
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
        use komple_types::query::ResponseWrapper;
        use komple_types::modules::marketplace::Listing;

        use super::*;

        mod fixed_tokens {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let hub_addr = setup_hub_module(&mut app, false);

                let (mint_module_addr, marketplace_module_addr) = setup_modules(&mut app, hub_addr);

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

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
                        &[],
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

                let (mint_module_addr, marketplace_module_addr) = setup_modules(&mut app, hub_addr);

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

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
                        &[],
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
        use komple_types::modules::marketplace::Listing;

        mod fixed_tokens {
            use cosmwasm_std::StdError;

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
                    .execute_contract(Addr::unchecked(ADMIN), mint_module_addr.clone(), &msg, &[])
                    .unwrap();

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    CREATOR,
                    token_module_code_id,
                );

                let collection_addr =
                    StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &1)
                        .unwrap();

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

                let locks =
                    StorageHelper::query_token_locks(&app.wrap(), &collection_addr, &1).unwrap();
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
                        &[coin(1_000, NATIVE_DENOM)],
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

                let locks =
                    StorageHelper::query_token_locks(&app.wrap(), &collection_addr, &1).unwrap();
                assert_eq!(locks.transfer_lock, false);
                assert_eq!(locks.send_lock, false);
                assert_eq!(locks.burn_lock, false);

                let owner =
                    StorageHelper::query_token_owner(&app.wrap(), &collection_addr, &1).unwrap();
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
                    msg: Some(
                        to_binary(&RegisterMsg {
                            admin: ADMIN.to_string(),
                            data: None,
                        })
                        .unwrap(),
                    ),
                    code_id: fee_module_code_id,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &[])
                    .unwrap();
                let fee_module_addr = StorageHelper::query_module_address(
                    &app.wrap(),
                    &hub_addr,
                    Modules::Fee.to_string(),
                )
                .unwrap();

                // Setup admin royalty for 10 percent
                set_royalties(&mut app, &fee_module_addr, 1, "0.1");

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
                        &[coin(1_000, NATIVE_DENOM)],
                    )
                    .unwrap();

                let owner =
                    StorageHelper::query_token_owner(&app.wrap(), &collection_addr, &1).unwrap();
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
                set_royalties(&mut app, &fee_module_addr, 1, "0.05");

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
                        &[coin(998_000, NATIVE_DENOM)],
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

            // #[test]
            // fn test_happy_path_without_marbu() {
            //     let mut app = mock_app();
            //     let hub_addr = setup_hub_module(&mut app, false);

            //     let (mint_module_addr, marketplace_module_addr) =
            //         setup_modules(&mut app, hub_addr.clone());

            //     // Register and setup fee module
            //     let fee_module_code_id = app.store_code(fee_module());
            //     let msg = HubExecuteMsg::RegisterModule {
            //         module: Modules::Fee.to_string(),
            //         msg: to_binary(&FeeModuleInstantiateMsg {
            //             admin: ADMIN.to_string(),
            //         })
            //         .unwrap(),
            //         code_id: fee_module_code_id,
            //     };
            //     let _ = app
            //         .execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &[])
            //         .unwrap();
            //     let fee_module_addr =
            //         StorageHelper::query_module_address(&app.wrap(), &hub_addr, Modules::Fee)
            //             .unwrap();
            //     setup_fee_module(&mut app, &fee_module_addr);

            //     // Update public permission settings
            //     // Creator will be creating the collection
            //     let msg = MintExecuteMsg::UpdatePublicCollectionCreation {
            //         public_collection_creation: true,
            //     };
            //     let _ = app
            //         .execute_contract(Addr::unchecked(ADMIN), mint_module_addr.clone(), &msg, &[])
            //         .unwrap();

            //     let token_module_code_id = app.store_code(token_module());
            //     create_collection(
            //         &mut app,
            //         mint_module_addr.clone(),
            //         CREATOR,
            //         token_module_code_id,
            //     );

            //     let collection_addr =
            //         StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &1)
            //             .unwrap();

            //     mint_token(&mut app, mint_module_addr.clone(), 1, USER);
            //     mint_token(&mut app, mint_module_addr.clone(), 1, USER);
            //     mint_token(&mut app, mint_module_addr.clone(), 1, USER);

            //     give_approval_to_module(
            //         &mut app,
            //         collection_addr.clone(),
            //         USER,
            //         &marketplace_module_addr,
            //     );

            //     setup_marketplace_listing(
            //         &mut app,
            //         &mint_module_addr,
            //         &marketplace_module_addr,
            //         1,
            //         1,
            //         Uint128::new(1_000),
            //     );

            //     let locks =
            //         StorageHelper::query_token_locks(&app.wrap(), &collection_addr, &1).unwrap();
            //     assert_eq!(locks.transfer_lock, true);
            //     assert_eq!(locks.send_lock, true);
            //     assert_eq!(locks.burn_lock, true);

            //     let msg = MarketplaceExecuteMsg::Buy {
            //         listing_type: Listing::Fixed,
            //         collection_id: 1,
            //         token_id: 1,
            //     };
            //     let _ = app
            //         .execute_contract(
            //             Addr::unchecked(RANDOM),
            //             marketplace_module_addr.clone(),
            //             &msg,
            //             &[coin(1_000, NATIVE_DENOM)],
            //         )
            //         .unwrap();

            //     let msg = MarketplaceQueryMsg::FixedListing {
            //         collection_id: 1,
            //         token_id: 1,
            //     };
            //     let res: Result<Empty, StdError> = app
            //         .wrap()
            //         .query_wasm_smart(marketplace_module_addr.clone(), &msg);
            //     assert!(res.is_err());

            //     let locks =
            //         StorageHelper::query_token_locks(&app.wrap(), &collection_addr, &1).unwrap();
            //     assert_eq!(locks.transfer_lock, false);
            //     assert_eq!(locks.send_lock, false);
            //     assert_eq!(locks.burn_lock, false);

            //     let owner =
            //         StorageHelper::query_token_owner(&app.wrap(), &collection_addr, &1).unwrap();
            //     assert_eq!(owner, Addr::unchecked(RANDOM));

            //     // Buyer balance
            //     let balance = app.wrap().query_balance(RANDOM, NATIVE_DENOM).unwrap();
            //     assert_eq!(balance.amount, Uint128::new(999_000));

            //     // Owner balance
            //     let balance = app.wrap().query_balance(USER, NATIVE_DENOM).unwrap();
            //     assert_eq!(balance.amount, Uint128::new(1_000_920));

            //     // Komple fee
            //     let balance = app.wrap().query_balance("contract0", NATIVE_DENOM).unwrap();
            //     assert_eq!(balance.amount, Uint128::new(40));

            //     // Community fee
            //     let balance = app
            //         .wrap()
            //         .query_balance("juno..community", NATIVE_DENOM)
            //         .unwrap();
            //     assert_eq!(balance.amount, Uint128::new(20));

            //     // Setup admin royalty for 10 percent
            //     set_royalties(&mut app, &fee_module_addr, &1, "0.1");

            //     setup_marketplace_listing(
            //         &mut app,
            //         &mint_module_addr,
            //         &marketplace_module_addr,
            //         1,
            //         2,
            //         Uint128::new(1_000),
            //     );

            //     let msg = MarketplaceExecuteMsg::Buy {
            //         listing_type: Listing::Fixed,
            //         collection_id: 1,
            //         token_id: 2,
            //     };
            //     let _ = app
            //         .execute_contract(
            //             Addr::unchecked(RANDOM),
            //             marketplace_module_addr.clone(),
            //             &msg,
            //             &[coin(1_000, NATIVE_DENOM)],
            //         )
            //         .unwrap();

            //     let owner =
            //         StorageHelper::query_token_owner(&app.wrap(), &collection_addr, &1).unwrap();
            //     assert_eq!(owner, Addr::unchecked(RANDOM));

            //     // Buyer balance
            //     let balance = app.wrap().query_balance(RANDOM, NATIVE_DENOM).unwrap();
            //     assert_eq!(balance.amount, Uint128::new(998_000));

            //     // Owner balance
            //     let balance = app.wrap().query_balance(USER, NATIVE_DENOM).unwrap();
            //     assert_eq!(balance.amount, Uint128::new(1_001_740));

            //     // Komple fee
            //     let balance = app.wrap().query_balance("contract0", NATIVE_DENOM).unwrap();
            //     assert_eq!(balance.amount, Uint128::new(80));

            //     // Community fee
            //     let balance = app
            //         .wrap()
            //         .query_balance("juno..community", NATIVE_DENOM)
            //         .unwrap();
            //     assert_eq!(balance.amount, Uint128::new(40));

            //     // Creator royalty fee
            //     let balance = app.wrap().query_balance(CREATOR, NATIVE_DENOM).unwrap();
            //     assert_eq!(balance.amount, Uint128::new(100));

            //     set_royalties(&mut app, &fee_module_addr, &1, "0.05");

            //     setup_marketplace_listing(
            //         &mut app,
            //         &mint_module_addr,
            //         &marketplace_module_addr,
            //         1,
            //         3,
            //         Uint128::new(998_000),
            //     );

            //     let msg = MarketplaceExecuteMsg::Buy {
            //         listing_type: Listing::Fixed,
            //         collection_id: 1,
            //         token_id: 3,
            //     };
            //     let _ = app
            //         .execute_contract(
            //             Addr::unchecked(RANDOM),
            //             marketplace_module_addr.clone(),
            //             &msg,
            //             &[coin(998_000, NATIVE_DENOM)],
            //         )
            //         .unwrap();

            //     // Buyer balance
            //     let balance = app.wrap().query_balance(RANDOM, NATIVE_DENOM).unwrap();
            //     assert_eq!(balance.amount, Uint128::new(0));

            //     // Owner balance
            //     let balance = app.wrap().query_balance(USER, NATIVE_DENOM).unwrap();
            //     assert_eq!(balance.amount, Uint128::new(1_870_000));

            //     // Komple fee
            //     let balance = app.wrap().query_balance("contract0", NATIVE_DENOM).unwrap();
            //     assert_eq!(balance.amount, Uint128::new(40_000));

            //     // Community fee
            //     let balance = app
            //         .wrap()
            //         .query_balance("juno..community", NATIVE_DENOM)
            //         .unwrap();
            //     assert_eq!(balance.amount, Uint128::new(20_000));

            //     // Creator royalty fee
            //     let balance = app.wrap().query_balance(CREATOR, NATIVE_DENOM).unwrap();
            //     assert_eq!(balance.amount, Uint128::new(50_000));
            // }

            #[test]
            fn test_invalid_funds() {
                let mut app = mock_app();

                let hub_addr = setup_hub_module(&mut app, false);

                let (mint_module_addr, marketplace_module_addr) = setup_modules(&mut app, hub_addr);

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

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
                        &[],
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
                        &[coin(1_000_000, TEST_DENOM)],
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
                        &[coin(100, NATIVE_DENOM)],
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

                let (mint_module_addr, marketplace_module_addr) = setup_modules(&mut app, hub_addr);

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    ADMIN,
                    token_module_code_id,
                );

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
                        &[],
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

        let (mint_module_addr, marketplace_module_addr) = setup_modules(&mut app, hub_addr);

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
