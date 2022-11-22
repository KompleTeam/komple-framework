use cosmwasm_std::{Addr, Empty};
use cw721_base::msg::{ExecuteMsg as Cw721ExecuteMsg, QueryMsg as Cw721QueryMsg};
use cw_multi_test::Executor;
use komple_framework_metadata_module::{
    msg::{InstantiateMsg as MetadataInstantiateMsg, QueryMsg as MetadataQueryMsg},
    state::{MetaInfo, Metadata as MetadataMetadata},
};
use komple_framework_mint_module::{
    msg::{CollectionFundInfo, ExecuteMsg as MintExecuteMsg},
    state::CollectionInfo,
};
use komple_framework_token_module::msg::{ExecuteMsg, MetadataInfo, QueryMsg, TokenInfo};
use komple_framework_token_module::state::{CollectionConfig, Config as TokenConfig};
use komple_framework_token_module::ContractError;
use komple_framework_types::modules::metadata::Metadata as MetadataType;
use komple_framework_types::modules::mint::Collections;
use komple_framework_types::modules::token::{Locks, SubModules};
use komple_framework_types::shared::query::ResponseWrapper;
use komple_framework_types::shared::RegisterMsg;
use komple_framework_utils::storage::StorageHelper;

pub mod helpers;
use helpers::*;

mod initialization {
    use super::*;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let mint_code_id = app.store_code(mint_module());
        let token_code_id = app.store_code(token_module());
        let metadata_code_id = app.store_code(metadata_module());

        let mint_module_addr = app
            .instantiate_contract(
                mint_code_id,
                Addr::unchecked(ADMIN),
                &RegisterMsg {
                    admin: ADMIN.to_string(),
                    data: None,
                },
                &vec![],
                "test",
                Some(ADMIN.to_string()),
            )
            .unwrap();

        let token_info = TokenInfo {
            symbol: "TTT".to_string(),
            minter: ADMIN.to_string(),
        };
        let collection_config = CollectionConfig {
            per_address_limit: Some(5),
            start_time: Some(app.block_info().time.plus_seconds(1)),
            max_token_limit: Some(100),
            ipfs_link: Some("some-link".to_string()),
        };
        let metadata_info = MetadataInfo {
            instantiate_msg: MetadataInstantiateMsg {
                metadata_type: MetadataType::Standard,
            },
            code_id: metadata_code_id,
        };

        let create_collection_msg = MintExecuteMsg::CreateCollection {
            code_id: token_code_id,
            collection_info: CollectionInfo {
                collection_type: Collections::Standard,
                name: "Test Collection".to_string(),
                description: "Test Description".to_string(),
                image: "ipfs://xyz".to_string(),
                external_link: None,
            },
            collection_config,
            fund_info: CollectionFundInfo {
                is_native: true,
                denom: NATIVE_DENOM.to_string(),
                cw20_address: None,
            },
            token_info,
            metadata_info,
            linked_collections: None,
        };
        app.execute_contract(
            Addr::unchecked(ADMIN),
            mint_module_addr.clone(),
            &create_collection_msg,
            &vec![],
        )
        .unwrap();

        let token_module_addr =
            StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

        assert_eq!(token_module_addr, "contract1");
    }

    #[test]
    fn test_invalid_time() {
        let mut app = mock_app();
        let mint_code_id = app.store_code(mint_module());
        let token_code_id = app.store_code(token_module());
        let metadata_code_id = app.store_code(metadata_module());

        let mint_module_addr = app
            .instantiate_contract(
                mint_code_id,
                Addr::unchecked(ADMIN),
                &RegisterMsg {
                    admin: ADMIN.to_string(),
                    data: None,
                },
                &vec![],
                "test",
                Some(ADMIN.to_string()),
            )
            .unwrap();

        let token_info = TokenInfo {
            symbol: "TTT".to_string(),
            minter: ADMIN.to_string(),
        };
        let mut collection_config = CollectionConfig {
            per_address_limit: Some(5),
            start_time: Some(app.block_info().time),
            max_token_limit: Some(100),
            ipfs_link: Some("some-link".to_string()),
        };
        let metadata_info = MetadataInfo {
            instantiate_msg: MetadataInstantiateMsg {
                metadata_type: MetadataType::Standard,
            },
            code_id: metadata_code_id,
        };
        let create_collection_msg = MintExecuteMsg::CreateCollection {
            code_id: token_code_id,
            collection_info: CollectionInfo {
                collection_type: Collections::Standard,
                name: "Test Collection".to_string(),
                description: "Test Description".to_string(),
                image: "ipfs://xyz".to_string(),
                external_link: None,
            },
            collection_config: collection_config.clone(),
            fund_info: CollectionFundInfo {
                is_native: true,
                denom: NATIVE_DENOM.to_string(),
                cw20_address: None,
            },
            token_info: token_info.clone(),
            metadata_info: metadata_info.clone(),
            linked_collections: None,
        };
        let err = app
            .execute_contract(
                Addr::unchecked(ADMIN),
                mint_module_addr.clone(),
                &create_collection_msg,
                &vec![],
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().source().unwrap().to_string(),
            ContractError::InvalidStartTime {}.to_string()
        );

        collection_config.start_time = Some(app.block_info().time.minus_seconds(10));
        let create_collection_msg = MintExecuteMsg::CreateCollection {
            code_id: token_code_id,
            collection_info: CollectionInfo {
                collection_type: Collections::Standard,
                name: "Test Collection".to_string(),
                description: "Test Description".to_string(),
                image: "ipfs://xyz".to_string(),
                external_link: None,
            },
            collection_config: collection_config,
            fund_info: CollectionFundInfo {
                is_native: true,
                denom: NATIVE_DENOM.to_string(),
                cw20_address: None,
            },
            token_info: token_info.clone(),
            metadata_info: metadata_info.clone(),
            linked_collections: None,
        };
        let err = app
            .execute_contract(
                Addr::unchecked(ADMIN),
                mint_module_addr.clone(),
                &create_collection_msg,
                &vec![],
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().source().unwrap().to_string(),
            ContractError::InvalidStartTime {}.to_string()
        );
    }

    #[test]
    fn test_invalid_max_token_limit() {
        let mut app = mock_app();
        let mint_code_id = app.store_code(mint_module());
        let token_code_id = app.store_code(token_module());
        let metadata_code_id = app.store_code(metadata_module());

        let mint_module_addr = app
            .instantiate_contract(
                mint_code_id,
                Addr::unchecked(ADMIN),
                &RegisterMsg {
                    admin: ADMIN.to_string(),
                    data: None,
                },
                &vec![],
                "test",
                Some(ADMIN.to_string()),
            )
            .unwrap();

        let token_info = TokenInfo {
            symbol: "TTT".to_string(),
            minter: ADMIN.to_string(),
        };
        let collection_config = CollectionConfig {
            per_address_limit: Some(5),
            start_time: Some(app.block_info().time.plus_seconds(1)),
            max_token_limit: Some(0),
            ipfs_link: Some("some-link".to_string()),
        };
        let metadata_info = MetadataInfo {
            instantiate_msg: MetadataInstantiateMsg {
                metadata_type: MetadataType::Standard,
            },
            code_id: metadata_code_id,
        };
        let create_collection_msg = MintExecuteMsg::CreateCollection {
            code_id: token_code_id,
            collection_info: CollectionInfo {
                collection_type: Collections::Standard,
                name: "Test Collection".to_string(),
                description: "Test Description".to_string(),
                image: "ipfs://xyz".to_string(),
                external_link: None,
            },
            collection_config: collection_config.clone(),
            fund_info: CollectionFundInfo {
                is_native: true,
                denom: NATIVE_DENOM.to_string(),
                cw20_address: None,
            },
            token_info: token_info.clone(),
            metadata_info: metadata_info.clone(),
            linked_collections: None,
        };
        let err = app
            .execute_contract(
                Addr::unchecked(ADMIN),
                mint_module_addr,
                &create_collection_msg,
                &vec![],
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().source().unwrap().to_string(),
            ContractError::InvalidMaxTokenLimit {}.to_string()
        );
    }

    #[test]
    fn test_invalid_per_address_limit() {
        let mut app = mock_app();
        let mint_code_id = app.store_code(mint_module());
        let token_code_id = app.store_code(token_module());
        let metadata_code_id = app.store_code(metadata_module());

        let mint_module_addr = app
            .instantiate_contract(
                mint_code_id,
                Addr::unchecked(ADMIN),
                &RegisterMsg {
                    admin: ADMIN.to_string(),
                    data: None,
                },
                &vec![],
                "test",
                Some(ADMIN.to_string()),
            )
            .unwrap();

        let token_info = TokenInfo {
            symbol: "TTT".to_string(),
            minter: ADMIN.to_string(),
        };
        let collection_config = CollectionConfig {
            per_address_limit: Some(0),
            start_time: Some(app.block_info().time.plus_seconds(1)),
            max_token_limit: Some(100),
            ipfs_link: Some("some-link".to_string()),
        };
        let metadata_info = MetadataInfo {
            instantiate_msg: MetadataInstantiateMsg {
                metadata_type: MetadataType::Standard,
            },
            code_id: metadata_code_id,
        };
        let create_collection_msg = MintExecuteMsg::CreateCollection {
            code_id: token_code_id,
            collection_info: CollectionInfo {
                collection_type: Collections::Standard,
                name: "Test Collection".to_string(),
                description: "Test Description".to_string(),
                image: "ipfs://xyz".to_string(),
                external_link: None,
            },
            collection_config: collection_config.clone(),
            fund_info: CollectionFundInfo {
                is_native: true,
                denom: NATIVE_DENOM.to_string(),
                cw20_address: None,
            },
            token_info: token_info.clone(),
            metadata_info: metadata_info.clone(),
            linked_collections: None,
        };
        let err = app
            .execute_contract(
                Addr::unchecked(ADMIN),
                mint_module_addr,
                &create_collection_msg,
                &vec![],
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().source().unwrap().to_string(),
            ContractError::InvalidPerAddressLimit {}.to_string()
        );
    }

    #[test]
    fn test_missing_ipfs_link() {
        let mut app = mock_app();
        let mint_code_id = app.store_code(mint_module());
        let token_code_id = app.store_code(token_module());
        let metadata_code_id = app.store_code(metadata_module());

        let mint_module_addr = app
            .instantiate_contract(
                mint_code_id,
                Addr::unchecked(ADMIN),
                &RegisterMsg {
                    admin: ADMIN.to_string(),
                    data: None,
                },
                &vec![],
                "test",
                Some(ADMIN.to_string()),
            )
            .unwrap();

        let token_info = TokenInfo {
            symbol: "TTT".to_string(),
            minter: ADMIN.to_string(),
        };
        let collection_config = CollectionConfig {
            per_address_limit: Some(5),
            start_time: Some(app.block_info().time.plus_seconds(1)),
            max_token_limit: Some(100),
            ipfs_link: None,
        };
        let metadata_info = MetadataInfo {
            instantiate_msg: MetadataInstantiateMsg {
                metadata_type: MetadataType::Standard,
            },
            code_id: metadata_code_id,
        };
        let create_collection_msg = MintExecuteMsg::CreateCollection {
            code_id: token_code_id,
            collection_info: CollectionInfo {
                collection_type: Collections::Standard,
                name: "Test Collection".to_string(),
                description: "Test Description".to_string(),
                image: "ipfs://xyz".to_string(),
                external_link: None,
            },
            collection_config: collection_config.clone(),
            fund_info: CollectionFundInfo {
                is_native: true,
                denom: NATIVE_DENOM.to_string(),
                cw20_address: None,
            },
            token_info: token_info.clone(),
            metadata_info: metadata_info.clone(),
            linked_collections: None,
        };
        let err = app
            .execute_contract(
                Addr::unchecked(ADMIN),
                mint_module_addr,
                &create_collection_msg,
                &vec![],
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().source().unwrap().to_string(),
            ContractError::IpfsNotFound {}.to_string()
        );
    }

    #[test]
    fn test_invalid_collection_metadata_type() {
        let mut app = mock_app();
        let mint_code_id = app.store_code(mint_module());
        let token_code_id = app.store_code(token_module());
        let metadata_code_id = app.store_code(metadata_module());

        let mint_module_addr = app
            .instantiate_contract(
                mint_code_id,
                Addr::unchecked(ADMIN),
                &RegisterMsg {
                    admin: ADMIN.to_string(),
                    data: None,
                },
                &vec![],
                "test",
                Some(ADMIN.to_string()),
            )
            .unwrap();

        let token_info = TokenInfo {
            symbol: "TTT".to_string(),
            minter: ADMIN.to_string(),
        };
        let collection_config = CollectionConfig {
            per_address_limit: Some(5),
            start_time: Some(app.block_info().time.plus_seconds(1)),
            max_token_limit: Some(100),
            ipfs_link: Some("some-link".to_string()),
        };
        let mut metadata_info = MetadataInfo {
            instantiate_msg: MetadataInstantiateMsg {
                metadata_type: MetadataType::Dynamic,
            },
            code_id: metadata_code_id,
        };
        let create_collection_msg = MintExecuteMsg::CreateCollection {
            code_id: token_code_id,
            collection_info: CollectionInfo {
                collection_type: Collections::Standard,
                name: "Test Collection".to_string(),
                description: "Test Description".to_string(),
                image: "ipfs://xyz".to_string(),
                external_link: None,
            },
            collection_config: collection_config.clone(),
            fund_info: CollectionFundInfo {
                is_native: true,
                denom: NATIVE_DENOM.to_string(),
                cw20_address: None,
            },
            token_info: token_info.clone(),
            metadata_info: metadata_info.clone(),
            linked_collections: None,
        };
        let err = app
            .execute_contract(
                Addr::unchecked(ADMIN),
                mint_module_addr.clone(),
                &create_collection_msg,
                &vec![],
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().source().unwrap().to_string(),
            ContractError::InvalidCollectionMetadataType {}.to_string()
        );

        metadata_info.instantiate_msg.metadata_type = MetadataType::Standard;
        let create_collection_msg = MintExecuteMsg::CreateCollection {
            code_id: token_code_id,
            collection_info: CollectionInfo {
                collection_type: Collections::Komple,
                name: "Test Collection".to_string(),
                description: "Test Description".to_string(),
                image: "ipfs://xyz".to_string(),
                external_link: None,
            },
            collection_config: collection_config.clone(),
            fund_info: CollectionFundInfo {
                is_native: true,
                denom: NATIVE_DENOM.to_string(),
                cw20_address: None,
            },
            token_info: token_info.clone(),
            metadata_info,
            linked_collections: None,
        };
        let err = app
            .execute_contract(
                Addr::unchecked(ADMIN),
                mint_module_addr,
                &create_collection_msg,
                &vec![],
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().source().unwrap().to_string(),
            ContractError::InvalidCollectionMetadataType {}.to_string()
        );
    }
}

mod actions {
    use super::*;

    mod update_operators {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let (_, token_module_addr) =
                proper_instantiate(&mut app, None, None, None, Some("some-link".to_string()));

            let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                msg: ExecuteMsg::UpdateModuleOperators {
                    addrs: vec![RANDOM.to_string(), RANDOM_2.to_string(), RANDOM.to_string()],
                },
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
                .unwrap();

            let msg = Cw721QueryMsg::Extension {
                msg: QueryMsg::ModuleOperators {},
            };
            let res: ResponseWrapper<Vec<String>> = app
                .wrap()
                .query_wasm_smart(token_module_addr, &msg)
                .unwrap();
            assert_eq!(res.data, vec![RANDOM.to_string(), RANDOM_2.to_string()]);
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let (_, token_module_addr) =
                proper_instantiate(&mut app, None, None, None, Some("some-link".to_string()));

            let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                msg: ExecuteMsg::UpdateModuleOperators {
                    addrs: vec![RANDOM.to_string(), RANDOM_2.to_string()],
                },
            };
            let err = app
                .execute_contract(Addr::unchecked(USER), token_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }
    }

    mod update_locks {
        use super::*;

        mod normal_locks {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let (_, token_module_addr) =
                    proper_instantiate(&mut app, None, None, None, Some("some-link".to_string()));

                let locks = Locks {
                    mint_lock: false,
                    burn_lock: true,
                    transfer_lock: true,
                    send_lock: false,
                };
                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::UpdateLocks {
                        locks: locks.clone(),
                    },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
                    .unwrap();

                let msg = Cw721QueryMsg::Extension {
                    msg: QueryMsg::Locks {},
                };
                let res: ResponseWrapper<Locks> = app
                    .wrap()
                    .query_wasm_smart(token_module_addr, &msg)
                    .unwrap();
                assert_eq!(res.data, locks);
            }

            #[test]
            fn test_invalid_admin() {
                let mut app = mock_app();
                let (_, token_module_addr) =
                    proper_instantiate(&mut app, None, None, None, Some("some-link".to_string()));

                let locks = Locks {
                    mint_lock: false,
                    burn_lock: true,
                    transfer_lock: true,
                    send_lock: false,
                };
                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::UpdateLocks { locks: locks },
                };
                let err = app
                    .execute_contract(Addr::unchecked(USER), token_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::Unauthorized {}.to_string()
                );
            }
        }

        mod token_locks {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let (mint_module_addr, token_module_addr) =
                    proper_instantiate(&mut app, None, None, None, Some("some-link".to_string()));

                let msg = MintExecuteMsg::Mint {
                    collection_id: 1,
                    metadata_id: None,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(USER), mint_module_addr.clone(), &msg, &[])
                    .unwrap();

                let locks = Locks {
                    mint_lock: false,
                    burn_lock: true,
                    transfer_lock: true,
                    send_lock: false,
                };
                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::UpdateTokenLocks {
                        token_id: "1".to_string(),
                        locks: locks.clone(),
                    },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
                    .unwrap();

                let msg = Cw721QueryMsg::Extension {
                    msg: QueryMsg::TokenLocks {
                        token_id: "1".to_string(),
                    },
                };
                let res: ResponseWrapper<Locks> = app
                    .wrap()
                    .query_wasm_smart(token_module_addr, &msg)
                    .unwrap();
                assert_eq!(res.data, locks);
            }

            #[test]
            fn test_invalid_admin() {
                let mut app = mock_app();
                let (_, token_module_addr) =
                    proper_instantiate(&mut app, None, None, None, Some("some-link".to_string()));

                let locks = Locks {
                    mint_lock: false,
                    burn_lock: true,
                    transfer_lock: true,
                    send_lock: false,
                };
                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::UpdateTokenLocks {
                        token_id: "1".to_string(),
                        locks,
                    },
                };
                let err = app
                    .execute_contract(Addr::unchecked(USER), token_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::Unauthorized {}.to_string()
                );
            }

            #[test]
            fn test_invalid_token_id() {
                let mut app = mock_app();
                let (_, token_module_addr) =
                    proper_instantiate(&mut app, None, None, None, Some("some-link".to_string()));

                let locks = Locks {
                    mint_lock: false,
                    burn_lock: true,
                    transfer_lock: true,
                    send_lock: false,
                };
                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::UpdateTokenLocks {
                        token_id: "1".to_string(),
                        locks,
                    },
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::TokenNotFound {}.to_string()
                );
            }
        }

        mod update_collection_config {
            use super::*;

            mod per_address_limit {
                use super::*;

                #[test]
                fn test_happy_path() {
                    let mut app = mock_app();
                    let (_, token_module_addr) = proper_instantiate(
                        &mut app,
                        None,
                        None,
                        None,
                        Some("some-link".to_string()),
                    );

                    let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                        msg: ExecuteMsg::UpdateCollectionConfig {
                            collection_config: CollectionConfig {
                                per_address_limit: Some(5),
                                start_time: None,
                                max_token_limit: None,
                                ipfs_link: None,
                            },
                        },
                    };
                    let _ = app
                        .execute_contract(
                            Addr::unchecked(ADMIN),
                            token_module_addr.clone(),
                            &msg,
                            &[],
                        )
                        .unwrap();

                    let msg = Cw721QueryMsg::Extension {
                        msg: QueryMsg::Config {},
                    };
                    let res: ResponseWrapper<TokenConfig> = app
                        .wrap()
                        .query_wasm_smart(token_module_addr, &msg)
                        .unwrap();
                    assert_eq!(res.data.per_address_limit, Some(5));
                }

                #[test]
                fn test_invalid_per_address_limit() {
                    let mut app = mock_app();
                    let (_, token_module_addr) = proper_instantiate(
                        &mut app,
                        None,
                        None,
                        None,
                        Some("some-link".to_string()),
                    );

                    let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                        msg: ExecuteMsg::UpdateCollectionConfig {
                            collection_config: CollectionConfig {
                                per_address_limit: Some(0),
                                start_time: None,
                                max_token_limit: None,
                                ipfs_link: None,
                            },
                        },
                    };
                    let err = app
                        .execute_contract(Addr::unchecked(ADMIN), token_module_addr, &msg, &[])
                        .unwrap_err();
                    assert_eq!(
                        err.source().unwrap().to_string(),
                        ContractError::InvalidPerAddressLimit {}.to_string()
                    );
                }
            }

            mod start_time {
                use super::*;

                #[test]
                fn test_happy_path() {
                    let mut app = mock_app();
                    let (_, token_module_addr) = proper_instantiate(
                        &mut app,
                        None,
                        None,
                        None,
                        Some("some-link".to_string()),
                    );

                    let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                        msg: ExecuteMsg::UpdateCollectionConfig {
                            collection_config: CollectionConfig {
                                per_address_limit: None,
                                start_time: Some(app.block_info().time.plus_seconds(5)),
                                max_token_limit: None,
                                ipfs_link: None,
                            },
                        },
                    };
                    let _ = app
                        .execute_contract(
                            Addr::unchecked(ADMIN),
                            token_module_addr.clone(),
                            &msg,
                            &[],
                        )
                        .unwrap();

                    let msg = Cw721QueryMsg::Extension {
                        msg: QueryMsg::Config {},
                    };
                    let res: ResponseWrapper<TokenConfig> = app
                        .wrap()
                        .query_wasm_smart(token_module_addr, &msg)
                        .unwrap();
                    assert_eq!(
                        res.data.start_time,
                        Some(app.block_info().time.plus_seconds(5))
                    );
                }

                #[test]
                fn test_invalid_time() {
                    let mut app = mock_app();
                    let start_time = app.block_info().time.plus_seconds(5);
                    let (_, token_module_addr) = proper_instantiate(
                        &mut app,
                        None,
                        Some(start_time),
                        None,
                        Some("some-link".to_string()),
                    );

                    let genesis_time = app.block_info().time;

                    app.update_block(|block| block.time = block.time.plus_seconds(10));

                    let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                        msg: ExecuteMsg::UpdateCollectionConfig {
                            collection_config: CollectionConfig {
                                per_address_limit: None,
                                start_time: Some(app.block_info().time.plus_seconds(5)),
                                max_token_limit: None,
                                ipfs_link: None,
                            },
                        },
                    };
                    let err = app
                        .execute_contract(
                            Addr::unchecked(ADMIN),
                            token_module_addr.clone(),
                            &msg,
                            &[],
                        )
                        .unwrap_err();
                    assert_eq!(
                        err.source().unwrap().to_string(),
                        ContractError::AlreadyStarted {}.to_string()
                    );

                    app.update_block(|block| block.time = block.time.minus_seconds(6));

                    let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                        msg: ExecuteMsg::UpdateCollectionConfig {
                            collection_config: CollectionConfig {
                                per_address_limit: None,
                                start_time: Some(genesis_time.plus_seconds(2)),
                                max_token_limit: None,
                                ipfs_link: None,
                            },
                        },
                    };
                    let err = app
                        .execute_contract(Addr::unchecked(ADMIN), token_module_addr, &msg, &[])
                        .unwrap_err();
                    assert_eq!(
                        err.source().unwrap().to_string(),
                        ContractError::InvalidStartTime {}.to_string()
                    );
                }
            }

            mod max_token_limit {
                use super::*;

                #[test]
                fn test_happy_path() {
                    let mut app = mock_app();
                    let (_, token_module_addr) = proper_instantiate(
                        &mut app,
                        None,
                        None,
                        None,
                        Some("some-link".to_string()),
                    );

                    let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                        msg: ExecuteMsg::UpdateCollectionConfig {
                            collection_config: CollectionConfig {
                                per_address_limit: None,
                                start_time: None,
                                max_token_limit: Some(5),
                                ipfs_link: None,
                            },
                        },
                    };
                    let _ = app
                        .execute_contract(
                            Addr::unchecked(ADMIN),
                            token_module_addr.clone(),
                            &msg,
                            &[],
                        )
                        .unwrap();

                    let msg = Cw721QueryMsg::Extension {
                        msg: QueryMsg::Config {},
                    };
                    let res: ResponseWrapper<TokenConfig> = app
                        .wrap()
                        .query_wasm_smart(token_module_addr, &msg)
                        .unwrap();
                    assert_eq!(res.data.max_token_limit, Some(5));
                }

                #[test]
                fn test_invalid_max_token_limit() {
                    let mut app = mock_app();
                    let (_, token_module_addr) = proper_instantiate(
                        &mut app,
                        None,
                        None,
                        None,
                        Some("some-link".to_string()),
                    );

                    let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                        msg: ExecuteMsg::UpdateCollectionConfig {
                            collection_config: CollectionConfig {
                                per_address_limit: None,
                                start_time: None,
                                max_token_limit: Some(0),
                                ipfs_link: None,
                            },
                        },
                    };
                    let err = app
                        .execute_contract(Addr::unchecked(ADMIN), token_module_addr, &msg, &[])
                        .unwrap_err();
                    assert_eq!(
                        err.source().unwrap().to_string(),
                        ContractError::InvalidMaxTokenLimit {}.to_string()
                    );
                }
            }

            mod ipfs_link {
                use super::*;

                #[test]
                fn test_happy_path() {
                    let mut app = mock_app();
                    let (_, token_module_addr) = proper_instantiate(
                        &mut app,
                        None,
                        None,
                        None,
                        Some("some-link".to_string()),
                    );

                    let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                        msg: ExecuteMsg::UpdateCollectionConfig {
                            collection_config: CollectionConfig {
                                per_address_limit: None,
                                start_time: None,
                                max_token_limit: None,
                                ipfs_link: Some("other-link".to_string()),
                            },
                        },
                    };
                    let _ = app
                        .execute_contract(
                            Addr::unchecked(ADMIN),
                            token_module_addr.clone(),
                            &msg,
                            &[],
                        )
                        .unwrap();

                    let msg = Cw721QueryMsg::Extension {
                        msg: QueryMsg::Config {},
                    };
                    let res: ResponseWrapper<TokenConfig> = app
                        .wrap()
                        .query_wasm_smart(token_module_addr, &msg)
                        .unwrap();
                    assert_eq!(res.data.ipfs_link, Some("other-link".to_string()));
                }
            }

            #[test]
            fn test_invalid_admin() {
                let mut app = mock_app();
                let (_, token_module_addr) =
                    proper_instantiate(&mut app, None, None, None, Some("some-link".to_string()));

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::UpdateCollectionConfig {
                        collection_config: CollectionConfig {
                            per_address_limit: Some(5),
                            start_time: None,
                            max_token_limit: None,
                            ipfs_link: None,
                        },
                    },
                };
                let err = app
                    .execute_contract(Addr::unchecked(USER), token_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::Unauthorized {}.to_string()
                );
            }
        }
    }

    mod operations {
        use super::*;

        mod transfer_operation {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let (mint_module_addr, token_module_addr) =
                    proper_instantiate(&mut app, None, None, None, Some("some-link".to_string()));

                let msg = MintExecuteMsg::Mint {
                    collection_id: 1,
                    metadata_id: None,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(USER), mint_module_addr.clone(), &msg, &[])
                    .unwrap();

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::TransferNft {
                        recipient: RANDOM.to_string(),
                        token_id: "1".to_string(),
                    },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(USER), token_module_addr.clone(), &msg, &[])
                    .unwrap();

                let res =
                    StorageHelper::query_token_owner(&app.wrap(), &token_module_addr, &1).unwrap();
                assert_eq!(res, Addr::unchecked(RANDOM));
            }

            #[test]
            fn test_invalid_locks() {
                let mut app = mock_app();
                let (mint_module_addr, token_module_addr) =
                    proper_instantiate(&mut app, None, None, None, Some("some-link".to_string()));

                let msg = MintExecuteMsg::Mint {
                    collection_id: 1,
                    metadata_id: None,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(USER), mint_module_addr.clone(), &msg, &[])
                    .unwrap();

                let locks = Locks {
                    transfer_lock: true,
                    mint_lock: false,
                    burn_lock: false,
                    send_lock: false,
                };

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::UpdateTokenLocks {
                        token_id: "1".to_string(),
                        locks: locks.clone(),
                    },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
                    .unwrap();

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::TransferNft {
                        recipient: RANDOM.to_string(),
                        token_id: "1".to_string(),
                    },
                };
                let err = app
                    .execute_contract(Addr::unchecked(USER), token_module_addr.clone(), &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::TransferLocked {}.to_string()
                );

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::UpdateLocks { locks: locks },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
                    .unwrap();

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::TransferNft {
                        recipient: RANDOM.to_string(),
                        token_id: "1".to_string(),
                    },
                };
                let err = app
                    .execute_contract(Addr::unchecked(USER), token_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::TransferLocked {}.to_string()
                );
            }
        }

        mod admin_transfer_operation {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let (mint_module_addr, token_module_addr) =
                    proper_instantiate(&mut app, None, None, None, Some("some-link".to_string()));

                let msg = MintExecuteMsg::Mint {
                    collection_id: 1,
                    metadata_id: None,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(USER), mint_module_addr.clone(), &msg, &[])
                    .unwrap();

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::ApproveAll {
                    operator: ADMIN.to_string(),
                    expires: None,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(USER), token_module_addr.clone(), &msg, &[])
                    .unwrap();

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::AdminTransferNft {
                        recipient: RANDOM.to_string(),
                        token_id: "1".to_string(),
                    },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
                    .unwrap();

                let res =
                    StorageHelper::query_token_owner(&app.wrap(), &token_module_addr, &1).unwrap();
                assert_eq!(res, Addr::unchecked(RANDOM));
            }

            #[test]
            fn test_invalid_admin() {
                let mut app = mock_app();
                let (mint_module_addr, token_module_addr) =
                    proper_instantiate(&mut app, None, None, None, Some("some-link".to_string()));

                let msg = MintExecuteMsg::Mint {
                    collection_id: 1,
                    metadata_id: None,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(USER), mint_module_addr, &msg, &[])
                    .unwrap();

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::AdminTransferNft {
                        recipient: RANDOM.to_string(),
                        token_id: "1".to_string(),
                    },
                };
                let err = app
                    .execute_contract(Addr::unchecked(USER), token_module_addr.clone(), &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::Unauthorized {}.to_string()
                );
            }
        }

        mod burn_operation {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let (mint_module_addr, token_module_addr) =
                    proper_instantiate(&mut app, None, None, None, Some("some-link".to_string()));

                let msg = MintExecuteMsg::Mint {
                    collection_id: 1,
                    metadata_id: None,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(USER), mint_module_addr.clone(), &msg, &[])
                    .unwrap();

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::Burn {
                        token_id: "1".to_string(),
                    },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(USER), token_module_addr.clone(), &msg, &[])
                    .unwrap();

                let res = StorageHelper::query_token_owner(&app.wrap(), &token_module_addr, &1);
                assert!(res.is_err());

                let msg = Cw721QueryMsg::Extension {
                    msg: QueryMsg::SubModules {},
                };
                let res: ResponseWrapper<SubModules> = app
                    .wrap()
                    .query_wasm_smart(token_module_addr, &msg)
                    .unwrap();

                let msg = MetadataQueryMsg::Metadata { token_id: 1 };
                let res: Result<Empty, cosmwasm_std::StdError> = app
                    .wrap()
                    .query_wasm_smart(res.data.metadata.unwrap(), &msg);
                assert!(res.is_err());
            }

            #[test]
            fn test_invalid_locks() {
                let mut app = mock_app();
                let (mint_module_addr, token_module_addr) =
                    proper_instantiate(&mut app, None, None, None, Some("some-link".to_string()));

                let msg = MintExecuteMsg::Mint {
                    collection_id: 1,
                    metadata_id: None,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(USER), mint_module_addr.clone(), &msg, &[])
                    .unwrap();

                let locks = Locks {
                    transfer_lock: false,
                    mint_lock: false,
                    burn_lock: true,
                    send_lock: false,
                };

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::UpdateTokenLocks {
                        token_id: "1".to_string(),
                        locks: locks.clone(),
                    },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
                    .unwrap();

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::Burn {
                        token_id: "1".to_string(),
                    },
                };
                let err = app
                    .execute_contract(Addr::unchecked(USER), token_module_addr.clone(), &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::BurnLocked {}.to_string()
                );

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::UpdateLocks { locks: locks },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
                    .unwrap();

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::Burn {
                        token_id: "1".to_string(),
                    },
                };
                let err = app
                    .execute_contract(Addr::unchecked(USER), token_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::BurnLocked {}.to_string()
                );
            }
        }

        mod mint_operation {
            use komple_framework_metadata_module::msg::MetadataResponse;

            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let (mint_module_addr, token_module_addr) =
                    proper_instantiate(&mut app, None, None, None, Some("some-link".to_string()));

                let msg = MintExecuteMsg::Mint {
                    collection_id: 1,
                    metadata_id: None,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(USER), mint_module_addr.clone(), &msg, &[])
                    .unwrap();

                let res =
                    StorageHelper::query_token_owner(&app.wrap(), &token_module_addr, &1).unwrap();
                assert_eq!(res, Addr::unchecked(USER));

                let msg = Cw721QueryMsg::Extension {
                    msg: QueryMsg::SubModules {},
                };
                let res: ResponseWrapper<SubModules> = app
                    .wrap()
                    .query_wasm_smart(token_module_addr, &msg)
                    .unwrap();

                let msg = MetadataQueryMsg::Metadata { token_id: 1 };
                let res: ResponseWrapper<MetadataResponse> = app
                    .wrap()
                    .query_wasm_smart(res.data.metadata.unwrap(), &msg)
                    .unwrap();
                assert_eq!(res.data.metadata_id, 1);
                assert_eq!(
                    res.data.metadata,
                    MetadataMetadata {
                        meta_info: MetaInfo {
                            image: Some("some-link/1".to_string()),
                            external_url: None,
                            description: None,
                            animation_url: None,
                            youtube_url: None
                        },
                        attributes: vec![]
                    }
                );
            }

            #[test]
            fn test_invalid_locks() {
                let mut app = mock_app();
                let (mint_module_addr, token_module_addr) =
                    proper_instantiate(&mut app, None, None, None, Some("some-link".to_string()));

                let locks = Locks {
                    mint_lock: true,
                    transfer_lock: false,
                    burn_lock: false,
                    send_lock: false,
                };

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::UpdateLocks { locks: locks },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
                    .unwrap();

                let msg = MintExecuteMsg::Mint {
                    collection_id: 1,
                    metadata_id: None,
                };
                let err = app
                    .execute_contract(Addr::unchecked(USER), mint_module_addr.clone(), &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().source().unwrap().to_string(),
                    ContractError::MintLocked {}.to_string()
                );
            }

            #[test]
            fn test_max_token_limit() {
                let mut app = mock_app();
                let (mint_module_addr, _) = proper_instantiate(
                    &mut app,
                    None,
                    None,
                    Some(2),
                    Some("some-link".to_string()),
                );

                let msg = MintExecuteMsg::Mint {
                    collection_id: 1,
                    metadata_id: None,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(USER), mint_module_addr.clone(), &msg, &[])
                    .unwrap();

                let msg = MintExecuteMsg::Mint {
                    collection_id: 1,
                    metadata_id: None,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(RANDOM), mint_module_addr.clone(), &msg, &[])
                    .unwrap();

                let msg = MintExecuteMsg::Mint {
                    collection_id: 1,
                    metadata_id: None,
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(RANDOM_2),
                        mint_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().source().unwrap().to_string(),
                    ContractError::TokenLimitReached {}.to_string()
                );
            }

            #[test]
            fn test_per_address_limit() {
                let mut app = mock_app();
                let (mint_module_addr, _) = proper_instantiate(
                    &mut app,
                    Some(2),
                    None,
                    None,
                    Some("some-link".to_string()),
                );

                let msg = MintExecuteMsg::Mint {
                    collection_id: 1,
                    metadata_id: None,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(USER), mint_module_addr.clone(), &msg, &[])
                    .unwrap();

                let msg = MintExecuteMsg::Mint {
                    collection_id: 1,
                    metadata_id: None,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(USER), mint_module_addr.clone(), &msg, &[])
                    .unwrap();

                let msg = MintExecuteMsg::Mint {
                    collection_id: 1,
                    metadata_id: None,
                };
                let err = app
                    .execute_contract(Addr::unchecked(USER), mint_module_addr.clone(), &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().source().unwrap().to_string(),
                    ContractError::TokenLimitReached {}.to_string()
                );
            }

            #[test]
            fn test_invalid_time() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(5);
                let (mint_module_addr, _) = proper_instantiate(
                    &mut app,
                    None,
                    Some(start_time),
                    None,
                    Some("some-link".to_string()),
                );

                let msg = MintExecuteMsg::Mint {
                    collection_id: 1,
                    metadata_id: None,
                };
                let err = app
                    .execute_contract(Addr::unchecked(USER), mint_module_addr.clone(), &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().source().unwrap().to_string(),
                    ContractError::MintingNotStarted {}.to_string()
                );
            }
        }
    }
}
