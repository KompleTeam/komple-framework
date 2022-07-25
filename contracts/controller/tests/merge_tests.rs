use controller_contract::msg::ExecuteMsg;
use cosmwasm_std::Addr;
use cw_multi_test::Executor;

pub mod helpers;
use helpers::{
    create_collection, get_modules_addresses, give_approval_to_module, merge_module, mint_token,
    mock_app, proper_instantiate, setup_all_modules, setup_mint_module_operators, token_contract,
    ADMIN, USER,
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
        let merge_module_code_id = app.store_code(merge_module());

        let msg = ExecuteMsg::InitMergeModule {
            code_id: merge_module_code_id,
        };
        let _ = app.execute_contract(
            Addr::unchecked(ADMIN),
            controller_addr.clone(),
            &msg,
            &vec![],
        );

        let res =
            query_module_address(&app.wrap(), &controller_addr, Modules::MergeModule).unwrap();
        assert_eq!(res, "contract1")
    }

    #[test]
    fn test_invalid_sender() {
        let mut app = mock_app();
        let controller_addr = proper_instantiate(&mut app);
        let merge_module_code_id = app.store_code(merge_module());

        let msg = ExecuteMsg::InitMergeModule {
            code_id: merge_module_code_id,
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

mod normal_merge {
    use super::*;

    use cosmwasm_std::to_binary;
    use cw721::OwnerOfResponse;
    use helpers::link_collection_to_collections;
    use merge_module::{
        msg::{ExecuteMsg as MergeExecuteMsg, MergeBurnMsg, MergeMsg},
        ContractError as MergeContractError,
    };
    use rift_types::collection::Collections;
    use rift_utils::query_collection_address;
    use token_contract::msg::QueryMsg as TokenQueryMsg;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let controller_addr = proper_instantiate(&mut app);

        setup_all_modules(&mut app, controller_addr.clone());

        let (mint_module_addr, merge_module_addr, _, _) =
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
        );

        link_collection_to_collections(&mut app, mint_module_addr.clone(), 2, vec![3]);

        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 3, USER);

        setup_mint_module_operators(
            &mut app,
            mint_module_addr.clone(),
            vec![merge_module_addr.to_string()],
        );

        let collection_1_addr =
            query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();
        give_approval_to_module(
            &mut app,
            collection_1_addr.clone(),
            USER,
            &merge_module_addr,
        );
        let collection_3_addr =
            query_collection_address(&app.wrap(), &mint_module_addr, &3).unwrap();
        give_approval_to_module(
            &mut app,
            collection_3_addr.clone(),
            USER,
            &merge_module_addr,
        );

        let merge_msg = MergeMsg {
            mint: vec![2],
            burn: vec![
                MergeBurnMsg {
                    collection_id: 1,
                    token_id: 1,
                },
                MergeBurnMsg {
                    collection_id: 1,
                    token_id: 3,
                },
                MergeBurnMsg {
                    collection_id: 3,
                    token_id: 1,
                },
            ],
        };
        let msg = MergeExecuteMsg::Merge {
            msg: to_binary(&merge_msg).unwrap(),
        };
        let _ = app
            .execute_contract(Addr::unchecked(USER), merge_module_addr, &msg, &vec![])
            .unwrap();

        let msg = TokenQueryMsg::OwnerOf {
            token_id: "1".to_string(),
            include_expired: None,
        };
        let res: Result<OwnerOfResponse, cosmwasm_std::StdError> =
            app.wrap().query_wasm_smart(collection_1_addr.clone(), &msg);
        assert!(res.is_err());

        let msg = TokenQueryMsg::OwnerOf {
            token_id: "3".to_string(),
            include_expired: None,
        };
        let res: Result<OwnerOfResponse, cosmwasm_std::StdError> =
            app.wrap().query_wasm_smart(collection_1_addr.clone(), &msg);
        assert!(res.is_err());

        let collection_2_addr =
            query_collection_address(&app.wrap(), &mint_module_addr, &2).unwrap();

        let msg = TokenQueryMsg::OwnerOf {
            token_id: "1".to_string(),
            include_expired: None,
        };
        let res: OwnerOfResponse = app
            .wrap()
            .query_wasm_smart(collection_2_addr.clone(), &msg)
            .unwrap();
        assert_eq!(res.owner, USER);
    }

    #[test]
    fn test_unhappy_path() {
        let mut app = mock_app();
        let controller_addr = proper_instantiate(&mut app);

        setup_all_modules(&mut app, controller_addr.clone());

        let (mint_module_addr, merge_module_addr, _, _) =
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
        );
        create_collection(
            &mut app,
            mint_module_addr.clone(),
            token_contract_code_id,
            None,
            None,
            Collections::Normal,
            Some(vec![2]),
            None,
        );

        mint_token(&mut app, mint_module_addr.clone(), 1, USER);

        let merge_msg = MergeMsg {
            mint: vec![2],
            burn: vec![],
        };
        let msg = MergeExecuteMsg::Merge {
            msg: to_binary(&merge_msg).unwrap(),
        };
        let err = app
            .execute_contract(
                Addr::unchecked(USER),
                merge_module_addr.clone(),
                &msg,
                &vec![],
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            MergeContractError::BurnNotFound {}.to_string()
        );

        let merge_msg = MergeMsg {
            mint: vec![3],
            burn: vec![MergeBurnMsg {
                collection_id: 1,
                token_id: 1,
            }],
        };
        let msg = MergeExecuteMsg::Merge {
            msg: to_binary(&merge_msg).unwrap(),
        };
        let err = app
            .execute_contract(
                Addr::unchecked(USER),
                merge_module_addr.clone(),
                &msg,
                &vec![],
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            MergeContractError::LinkedCollectionNotFound {}.to_string()
        );

        let merge_msg = MergeMsg {
            mint: vec![2],
            burn: vec![MergeBurnMsg {
                collection_id: 1,
                token_id: 1,
            }],
        };
        let msg = MergeExecuteMsg::Merge {
            msg: to_binary(&merge_msg).unwrap(),
        };
        let err = app
            .execute_contract(
                Addr::unchecked(USER),
                merge_module_addr.clone(),
                &msg,
                &vec![],
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().source().unwrap().to_string(),
            MergeContractError::Unauthorized {}.to_string()
        );

        setup_mint_module_operators(
            &mut app,
            mint_module_addr.clone(),
            vec![merge_module_addr.to_string()],
        );

        let err = app
            .execute_contract(
                Addr::unchecked(USER),
                merge_module_addr.clone(),
                &msg,
                &vec![],
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().source().unwrap().to_string(),
            MergeContractError::Unauthorized {}.to_string()
        );

        setup_mint_module_operators(&mut app, mint_module_addr.clone(), vec![]);
        let collection_1_addr =
            query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();
        give_approval_to_module(
            &mut app,
            collection_1_addr.clone(),
            USER,
            &merge_module_addr,
        );

        let err = app
            .execute_contract(
                Addr::unchecked(USER),
                merge_module_addr.clone(),
                &msg,
                &vec![],
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().source().unwrap().to_string(),
            MergeContractError::Unauthorized {}.to_string()
        );
    }
}

mod permission_merge {
    use super::*;

    use cosmwasm_std::to_binary;
    use cw721::OwnerOfResponse;
    use helpers::{add_permission_for_module, link_collection_to_collections};
    use merge_module::msg::{ExecuteMsg as MergeExecuteMsg, MergeBurnMsg, MergeMsg};
    use permission_module::msg::PermissionCheckMsg;
    use rift_types::collection::Collections;
    use rift_types::module::Modules;
    use rift_types::permission::Permissions;
    use token_contract::msg::QueryMsg as TokenQueryMsg;

    mod ownership_permission {
        use super::*;

        use cosmwasm_std::coin;
        use permission_module::msg::OwnershipMsg;
        use rift_utils::query_collection_address;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let controller_addr = proper_instantiate(&mut app);

            setup_all_modules(&mut app, controller_addr.clone());

            let (mint_module_addr, merge_module_addr, permission_module_addr, _) =
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
            );

            link_collection_to_collections(&mut app, mint_module_addr.clone(), 2, vec![3]);

            mint_token(&mut app, mint_module_addr.clone(), 1, USER);
            mint_token(&mut app, mint_module_addr.clone(), 1, USER);
            mint_token(&mut app, mint_module_addr.clone(), 1, USER);
            mint_token(&mut app, mint_module_addr.clone(), 3, USER);

            setup_mint_module_operators(
                &mut app,
                mint_module_addr.clone(),
                vec![merge_module_addr.to_string()],
            );

            let collection_1_addr =
                query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();
            give_approval_to_module(
                &mut app,
                collection_1_addr.clone(),
                USER,
                &merge_module_addr,
            );
            let collection_3_addr =
                query_collection_address(&app.wrap(), &mint_module_addr, &3).unwrap();
            give_approval_to_module(
                &mut app,
                collection_3_addr.clone(),
                USER,
                &merge_module_addr,
            );

            add_permission_for_module(
                &mut app,
                permission_module_addr,
                Modules::MergeModule,
                vec![Permissions::Ownership],
            );

            let permission_msg = to_binary(&vec![PermissionCheckMsg {
                permission_type: Permissions::Ownership,
                data: to_binary(&vec![
                    OwnershipMsg {
                        collection_id: 1,
                        token_id: 1,
                        owner: USER.to_string(),
                    },
                    OwnershipMsg {
                        collection_id: 1,
                        token_id: 2,
                        owner: USER.to_string(),
                    },
                ])
                .unwrap(),
            }])
            .unwrap();
            let merge_msg = to_binary(&MergeMsg {
                mint: vec![2],
                burn: vec![
                    MergeBurnMsg {
                        collection_id: 1,
                        token_id: 1,
                    },
                    MergeBurnMsg {
                        collection_id: 1,
                        token_id: 3,
                    },
                    MergeBurnMsg {
                        collection_id: 3,
                        token_id: 1,
                    },
                ],
            })
            .unwrap();
            let msg = MergeExecuteMsg::PermissionMerge {
                permission_msg,
                merge_msg,
            };
            let _ = app
                .execute_contract(Addr::unchecked(USER), merge_module_addr, &msg, &vec![])
                .unwrap();

            let msg = TokenQueryMsg::OwnerOf {
                token_id: "1".to_string(),
                include_expired: None,
            };
            let res: Result<OwnerOfResponse, cosmwasm_std::StdError> =
                app.wrap().query_wasm_smart(collection_1_addr.clone(), &msg);
            assert!(res.is_err());

            let msg = TokenQueryMsg::OwnerOf {
                token_id: "3".to_string(),
                include_expired: None,
            };
            let res: Result<OwnerOfResponse, cosmwasm_std::StdError> =
                app.wrap().query_wasm_smart(collection_1_addr.clone(), &msg);
            assert!(res.is_err());

            let collection_2_addr =
                query_collection_address(&app.wrap(), &mint_module_addr, &2).unwrap();

            let msg = TokenQueryMsg::OwnerOf {
                token_id: "1".to_string(),
                include_expired: None,
            };
            let res: OwnerOfResponse = app
                .wrap()
                .query_wasm_smart(collection_2_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.owner, USER);
        }
    }
}
