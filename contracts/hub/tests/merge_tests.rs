use cosmwasm_std::Addr;
use cw_multi_test::Executor;
use komple_hub_module::msg::ExecuteMsg;

pub mod helpers;
use helpers::{
    create_collection, get_modules_addresses, give_approval_to_module, merge_module, mint_token,
    mock_app, proper_instantiate, setup_all_modules, setup_fee_contract, setup_metadata,
    setup_metadata_module, setup_mint_module_operators, token_module, ADMIN, USER,
};

mod initialization {
    use super::*;

    use komple_types::module::Modules;

    use komple_hub_module::ContractError;
    use komple_utils::query_module_address;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let hub_addr = proper_instantiate(&mut app);
        let merge_module_code_id = app.store_code(merge_module());

        let msg = ExecuteMsg::InitMergeModule {
            code_id: merge_module_code_id,
        };
        let _ = app.execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &vec![]);

        let res = query_module_address(&app.wrap(), &hub_addr, Modules::Merge).unwrap();
        assert_eq!(res, "contract1")
    }

    #[test]
    fn test_invalid_sender() {
        let mut app = mock_app();
        let hub_addr = proper_instantiate(&mut app);
        let merge_module_code_id = app.store_code(merge_module());

        let msg = ExecuteMsg::InitMergeModule {
            code_id: merge_module_code_id,
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

mod normal_merge {
    use super::*;

    use cosmwasm_std::to_binary;
    use cw721::OwnerOfResponse;
    use helpers::link_collections;
    use komple_merge_module::{
        msg::{ExecuteMsg as MergeExecuteMsg, MergeBurnMsg, MergeMsg},
        ContractError as MergeContractError,
    };
    use komple_token_module::msg::QueryMsg as TokenQueryMsg;
    use komple_types::{collection::Collections, metadata::Metadata};
    use komple_utils::query_collection_address;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        setup_fee_contract(&mut app);
        let hub_addr = proper_instantiate(&mut app);

        setup_all_modules(&mut app, hub_addr.clone());

        let (mint_module_addr, merge_module_addr, _, _) =
            get_modules_addresses(&mut app, &hub_addr);

        let token_module_code_id = app.store_code(token_module());
        create_collection(
            &mut app,
            mint_module_addr.clone(),
            token_module_code_id,
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
            token_module_code_id,
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
            token_module_code_id,
            None,
            None,
            Collections::Normal,
            None,
            None,
            None,
            None,
        );

        link_collections(&mut app, mint_module_addr.clone(), 2, vec![3]);

        let collection_1_addr =
            query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();
        let collection_2_addr =
            query_collection_address(&app.wrap(), &mint_module_addr, &2).unwrap();
        let collection_3_addr =
            query_collection_address(&app.wrap(), &mint_module_addr, &3).unwrap();

        let metadata_module_addr_1 =
            setup_metadata_module(&mut app, collection_1_addr.clone(), Metadata::Standard);
        let metadata_module_addr_2 =
            setup_metadata_module(&mut app, collection_2_addr.clone(), Metadata::Standard);
        let metadata_module_addr_3 =
            setup_metadata_module(&mut app, collection_3_addr.clone(), Metadata::Standard);

        setup_metadata(&mut app, metadata_module_addr_1.clone());
        setup_metadata(&mut app, metadata_module_addr_1.clone());
        setup_metadata(&mut app, metadata_module_addr_1.clone());
        setup_metadata(&mut app, metadata_module_addr_2);
        setup_metadata(&mut app, metadata_module_addr_3);

        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 3, USER);

        setup_mint_module_operators(
            &mut app,
            mint_module_addr.clone(),
            vec![merge_module_addr.to_string()],
        );

        give_approval_to_module(
            &mut app,
            collection_1_addr.clone(),
            USER,
            &merge_module_addr,
        );
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
            metadata_ids: None,
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
        setup_fee_contract(&mut app);
        let hub_addr = proper_instantiate(&mut app);

        setup_all_modules(&mut app, hub_addr.clone());

        let (mint_module_addr, merge_module_addr, _, _) =
            get_modules_addresses(&mut app, &hub_addr);

        let token_module_code_id = app.store_code(token_module());
        create_collection(
            &mut app,
            mint_module_addr.clone(),
            token_module_code_id,
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
            token_module_code_id,
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
            token_module_code_id,
            None,
            None,
            Collections::Normal,
            Some(vec![2]),
            None,
            None,
            None,
        );

        let collection_1_addr =
            query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();
        let metadata_module_addr_1 =
            setup_metadata_module(&mut app, collection_1_addr.clone(), Metadata::Standard);
        setup_metadata(&mut app, metadata_module_addr_1.clone());

        mint_token(&mut app, mint_module_addr.clone(), 1, USER);

        let merge_msg = MergeMsg {
            mint: vec![2],
            burn: vec![],
            metadata_ids: None,
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
            metadata_ids: None,
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
            metadata_ids: None,
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
    use helpers::{add_permission_for_module, link_collections};
    use komple_merge_module::msg::{ExecuteMsg as MergeExecuteMsg, MergeBurnMsg, MergeMsg};
    use komple_permission_module::msg::PermissionCheckMsg;
    use komple_token_module::msg::QueryMsg as TokenQueryMsg;
    use komple_types::collection::Collections;
    use komple_types::module::Modules;
    use komple_types::permission::Permissions;

    mod ownership_permission {
        use super::*;

        use komple_permission_module::msg::OwnershipMsg;
        use komple_types::metadata::Metadata;
        use komple_utils::query_collection_address;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            setup_fee_contract(&mut app);
            let hub_addr = proper_instantiate(&mut app);

            setup_all_modules(&mut app, hub_addr.clone());

            let (mint_module_addr, merge_module_addr, permission_module_addr, _) =
                get_modules_addresses(&mut app, &hub_addr);

            let token_module_code_id = app.store_code(token_module());
            create_collection(
                &mut app,
                mint_module_addr.clone(),
                token_module_code_id,
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
                token_module_code_id,
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
                token_module_code_id,
                None,
                None,
                Collections::Normal,
                None,
                None,
                None,
                None,
            );

            link_collections(&mut app, mint_module_addr.clone(), 2, vec![3]);

            let collection_1_addr =
                query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();
            let collection_2_addr =
                query_collection_address(&app.wrap(), &mint_module_addr, &2).unwrap();
            let collection_3_addr =
                query_collection_address(&app.wrap(), &mint_module_addr, &3).unwrap();

            let metadata_module_addr_1 =
                setup_metadata_module(&mut app, collection_1_addr.clone(), Metadata::Standard);
            let metadata_module_addr_2 =
                setup_metadata_module(&mut app, collection_2_addr.clone(), Metadata::Standard);
            let metadata_module_addr_3 =
                setup_metadata_module(&mut app, collection_3_addr.clone(), Metadata::Standard);

            setup_metadata(&mut app, metadata_module_addr_1.clone());
            setup_metadata(&mut app, metadata_module_addr_1.clone());
            setup_metadata(&mut app, metadata_module_addr_1.clone());
            setup_metadata(&mut app, metadata_module_addr_2);
            setup_metadata(&mut app, metadata_module_addr_3);

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
                Modules::Merge,
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
                metadata_ids: None,
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