use collection_contract::msg::ExecuteMsg;
use cosmwasm_std::Addr;
use cw_multi_test::Executor;

pub mod helpers;
use helpers::{
    create_bundle, get_modules_addresses, give_approval_to_module, merge_module, mint_token,
    mock_app, proper_instantiate, setup_all_modules, setup_fee_contract, setup_metadata,
    setup_metadata_contract, setup_mint_module_operators, token_contract, ADMIN, USER,
};

mod initialization {
    use super::*;

    use komple_types::module::Modules;

    use collection_contract::ContractError;
    use komple_utils::query_module_address;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let collection_addr = proper_instantiate(&mut app);
        let merge_module_code_id = app.store_code(merge_module());

        let msg = ExecuteMsg::InitMergeModule {
            code_id: merge_module_code_id,
        };
        let _ = app.execute_contract(
            Addr::unchecked(ADMIN),
            collection_addr.clone(),
            &msg,
            &vec![],
        );

        let res =
            query_module_address(&app.wrap(), &collection_addr, Modules::MergeModule).unwrap();
        assert_eq!(res, "contract1")
    }

    #[test]
    fn test_invalid_sender() {
        let mut app = mock_app();
        let collection_addr = proper_instantiate(&mut app);
        let merge_module_code_id = app.store_code(merge_module());

        let msg = ExecuteMsg::InitMergeModule {
            code_id: merge_module_code_id,
        };
        let err = app
            .execute_contract(
                Addr::unchecked(USER),
                collection_addr.clone(),
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
    use helpers::link_bundles;
    use komple_types::{bundle::Bundles, metadata::Metadata};
    use komple_utils::query_bundle_address;
    use merge_module::{
        msg::{ExecuteMsg as MergeExecuteMsg, MergeBurnMsg, MergeMsg},
        ContractError as MergeContractError,
    };
    use token_contract::msg::QueryMsg as TokenQueryMsg;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        setup_fee_contract(&mut app);
        let collection_addr = proper_instantiate(&mut app);

        setup_all_modules(&mut app, collection_addr.clone());

        let (mint_module_addr, merge_module_addr, _, _) =
            get_modules_addresses(&mut app, &collection_addr);

        let token_contract_code_id = app.store_code(token_contract());
        create_bundle(
            &mut app,
            mint_module_addr.clone(),
            token_contract_code_id,
            None,
            None,
            Bundles::Normal,
            None,
            None,
            None,
            None,
        );
        create_bundle(
            &mut app,
            mint_module_addr.clone(),
            token_contract_code_id,
            None,
            None,
            Bundles::Normal,
            None,
            None,
            None,
            None,
        );
        create_bundle(
            &mut app,
            mint_module_addr.clone(),
            token_contract_code_id,
            None,
            None,
            Bundles::Normal,
            None,
            None,
            None,
            None,
        );

        link_bundles(&mut app, mint_module_addr.clone(), 2, vec![3]);

        let bundle_1_addr =
            query_bundle_address(&app.wrap(), &mint_module_addr, &1).unwrap();
        let bundle_2_addr =
            query_bundle_address(&app.wrap(), &mint_module_addr, &2).unwrap();
        let bundle_3_addr =
            query_bundle_address(&app.wrap(), &mint_module_addr, &3).unwrap();

        let metadata_contract_addr_1 =
            setup_metadata_contract(&mut app, bundle_1_addr.clone(), Metadata::OneToOne);
        let metadata_contract_addr_2 =
            setup_metadata_contract(&mut app, bundle_2_addr.clone(), Metadata::OneToOne);
        let metadata_contract_addr_3 =
            setup_metadata_contract(&mut app, bundle_3_addr.clone(), Metadata::OneToOne);

        setup_metadata(&mut app, metadata_contract_addr_1.clone());
        setup_metadata(&mut app, metadata_contract_addr_1.clone());
        setup_metadata(&mut app, metadata_contract_addr_1.clone());
        setup_metadata(&mut app, metadata_contract_addr_2);
        setup_metadata(&mut app, metadata_contract_addr_3);

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
            bundle_1_addr.clone(),
            USER,
            &merge_module_addr,
        );
        give_approval_to_module(
            &mut app,
            bundle_3_addr.clone(),
            USER,
            &merge_module_addr,
        );

        let merge_msg = MergeMsg {
            mint: vec![2],
            burn: vec![
                MergeBurnMsg {
                    bundle_id: 1,
                    token_id: 1,
                },
                MergeBurnMsg {
                    bundle_id: 1,
                    token_id: 3,
                },
                MergeBurnMsg {
                    bundle_id: 3,
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
            app.wrap().query_wasm_smart(bundle_1_addr.clone(), &msg);
        assert!(res.is_err());

        let msg = TokenQueryMsg::OwnerOf {
            token_id: "3".to_string(),
            include_expired: None,
        };
        let res: Result<OwnerOfResponse, cosmwasm_std::StdError> =
            app.wrap().query_wasm_smart(bundle_1_addr.clone(), &msg);
        assert!(res.is_err());

        let bundle_2_addr =
            query_bundle_address(&app.wrap(), &mint_module_addr, &2).unwrap();

        let msg = TokenQueryMsg::OwnerOf {
            token_id: "1".to_string(),
            include_expired: None,
        };
        let res: OwnerOfResponse = app
            .wrap()
            .query_wasm_smart(bundle_2_addr.clone(), &msg)
            .unwrap();
        assert_eq!(res.owner, USER);
    }

    #[test]
    fn test_unhappy_path() {
        let mut app = mock_app();
        setup_fee_contract(&mut app);
        let collection_addr = proper_instantiate(&mut app);

        setup_all_modules(&mut app, collection_addr.clone());

        let (mint_module_addr, merge_module_addr, _, _) =
            get_modules_addresses(&mut app, &collection_addr);

        let token_contract_code_id = app.store_code(token_contract());
        create_bundle(
            &mut app,
            mint_module_addr.clone(),
            token_contract_code_id,
            None,
            None,
            Bundles::Normal,
            None,
            None,
            None,
            None,
        );
        create_bundle(
            &mut app,
            mint_module_addr.clone(),
            token_contract_code_id,
            None,
            None,
            Bundles::Normal,
            None,
            None,
            None,
            None,
        );
        create_bundle(
            &mut app,
            mint_module_addr.clone(),
            token_contract_code_id,
            None,
            None,
            Bundles::Normal,
            Some(vec![2]),
            None,
            None,
            None,
        );

        let bundle_1_addr =
            query_bundle_address(&app.wrap(), &mint_module_addr, &1).unwrap();
        let metadata_contract_addr_1 =
            setup_metadata_contract(&mut app, bundle_1_addr.clone(), Metadata::OneToOne);
        setup_metadata(&mut app, metadata_contract_addr_1.clone());

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
                bundle_id: 1,
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
            MergeContractError::LinkedBundleNotFound {}.to_string()
        );

        let merge_msg = MergeMsg {
            mint: vec![2],
            burn: vec![MergeBurnMsg {
                bundle_id: 1,
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
        let bundle_1_addr =
            query_bundle_address(&app.wrap(), &mint_module_addr, &1).unwrap();
        give_approval_to_module(
            &mut app,
            bundle_1_addr.clone(),
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
    use helpers::{add_permission_for_module, link_bundles};
    use komple_types::bundle::Bundles;
    use komple_types::module::Modules;
    use komple_types::permission::Permissions;
    use merge_module::msg::{ExecuteMsg as MergeExecuteMsg, MergeBurnMsg, MergeMsg};
    use permission_module::msg::PermissionCheckMsg;
    use token_contract::msg::QueryMsg as TokenQueryMsg;

    mod ownership_permission {
        use super::*;

        use komple_types::metadata::Metadata;
        use komple_utils::query_bundle_address;
        use permission_module::msg::OwnershipMsg;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            setup_fee_contract(&mut app);
            let collection_addr = proper_instantiate(&mut app);

            setup_all_modules(&mut app, collection_addr.clone());

            let (mint_module_addr, merge_module_addr, permission_module_addr, _) =
                get_modules_addresses(&mut app, &collection_addr);

            let token_contract_code_id = app.store_code(token_contract());
            create_bundle(
                &mut app,
                mint_module_addr.clone(),
                token_contract_code_id,
                None,
                None,
                Bundles::Normal,
                None,
                None,
                None,
                None,
            );
            create_bundle(
                &mut app,
                mint_module_addr.clone(),
                token_contract_code_id,
                None,
                None,
                Bundles::Normal,
                None,
                None,
                None,
                None,
            );
            create_bundle(
                &mut app,
                mint_module_addr.clone(),
                token_contract_code_id,
                None,
                None,
                Bundles::Normal,
                None,
                None,
                None,
                None,
            );

            link_bundles(&mut app, mint_module_addr.clone(), 2, vec![3]);

            let bundle_1_addr =
                query_bundle_address(&app.wrap(), &mint_module_addr, &1).unwrap();
            let bundle_2_addr =
                query_bundle_address(&app.wrap(), &mint_module_addr, &2).unwrap();
            let bundle_3_addr =
                query_bundle_address(&app.wrap(), &mint_module_addr, &3).unwrap();

            let metadata_contract_addr_1 =
                setup_metadata_contract(&mut app, bundle_1_addr.clone(), Metadata::OneToOne);
            let metadata_contract_addr_2 =
                setup_metadata_contract(&mut app, bundle_2_addr.clone(), Metadata::OneToOne);
            let metadata_contract_addr_3 =
                setup_metadata_contract(&mut app, bundle_3_addr.clone(), Metadata::OneToOne);

            setup_metadata(&mut app, metadata_contract_addr_1.clone());
            setup_metadata(&mut app, metadata_contract_addr_1.clone());
            setup_metadata(&mut app, metadata_contract_addr_1.clone());
            setup_metadata(&mut app, metadata_contract_addr_2);
            setup_metadata(&mut app, metadata_contract_addr_3);

            mint_token(&mut app, mint_module_addr.clone(), 1, USER);
            mint_token(&mut app, mint_module_addr.clone(), 1, USER);
            mint_token(&mut app, mint_module_addr.clone(), 1, USER);
            mint_token(&mut app, mint_module_addr.clone(), 3, USER);

            setup_mint_module_operators(
                &mut app,
                mint_module_addr.clone(),
                vec![merge_module_addr.to_string()],
            );

            let bundle_1_addr =
                query_bundle_address(&app.wrap(), &mint_module_addr, &1).unwrap();
            give_approval_to_module(
                &mut app,
                bundle_1_addr.clone(),
                USER,
                &merge_module_addr,
            );
            let bundle_3_addr =
                query_bundle_address(&app.wrap(), &mint_module_addr, &3).unwrap();
            give_approval_to_module(
                &mut app,
                bundle_3_addr.clone(),
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
                        bundle_id: 1,
                        token_id: 1,
                        owner: USER.to_string(),
                    },
                    OwnershipMsg {
                        bundle_id: 1,
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
                        bundle_id: 1,
                        token_id: 1,
                    },
                    MergeBurnMsg {
                        bundle_id: 1,
                        token_id: 3,
                    },
                    MergeBurnMsg {
                        bundle_id: 3,
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
                app.wrap().query_wasm_smart(bundle_1_addr.clone(), &msg);
            assert!(res.is_err());

            let msg = TokenQueryMsg::OwnerOf {
                token_id: "3".to_string(),
                include_expired: None,
            };
            let res: Result<OwnerOfResponse, cosmwasm_std::StdError> =
                app.wrap().query_wasm_smart(bundle_1_addr.clone(), &msg);
            assert!(res.is_err());

            let bundle_2_addr =
                query_bundle_address(&app.wrap(), &mint_module_addr, &2).unwrap();

            let msg = TokenQueryMsg::OwnerOf {
                token_id: "1".to_string(),
                include_expired: None,
            };
            let res: OwnerOfResponse = app
                .wrap()
                .query_wasm_smart(bundle_2_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.owner, USER);
        }
    }
}
