use controller_contract::msg::ExecuteMsg;
use cosmwasm_std::Addr;
use cw_multi_test::Executor;

pub mod helpers;
use helpers::{
    create_collection, get_modules_addresses, marketplace_module, mint_token, mock_app,
    proper_instantiate, setup_all_modules, token_contract, ADMIN, USER,
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

mod listing {
    use super::*;

    use cosmwasm_std::Uint128;
    use marketplace_module::msg::{
        ExecuteMsg as MarketplaceExecuteMsg, QueryMsg as MarketplaceQueryMsg,
    };
    use rift_types::collection::Collections;
    use token_contract::{msg::ExecuteMsg as TokenExecuteMsg, state::Contracts};

    mod fixed_tokens {
        use super::*;

        use marketplace_module::state::FixedListing;
        use rift_types::{query::ResponseWrapper, tokens::Locks};
        use rift_utils::query_collection_address;
        use token_contract::ContractError as TokenContractError;

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
                Contracts {
                    whitelist: None,
                    royalty: None,
                    metadata: None,
                },
            );

            mint_token(&mut app, mint_module_addr.clone(), 1, USER);

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
                Contracts {
                    whitelist: None,
                    royalty: None,
                    metadata: None,
                },
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
    }
}
