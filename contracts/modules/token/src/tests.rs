use crate::msg::{ExecuteMsg, InstantiateMsg, MetadataInfo, QueryMsg, TokenInfo};
use crate::state::{CollectionConfig, Config as TokenConfig};
use crate::ContractError;
use cosmwasm_std::{coin, to_binary, Addr, Coin, Empty, Timestamp, Uint128};
use cw721_base::msg::{ExecuteMsg as Cw721ExecuteMsg, QueryMsg as Cw721QueryMsg};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_metadata_module::{
    msg::{InstantiateMsg as MetadataInstantiateMsg, QueryMsg as MetadataQueryMsg},
    state::{MetaInfo, Metadata as MetadataMetadata},
};
use komple_types::shared::RegisterMsg;
use komple_types::{
    mint::Collections,
    metadata::Metadata as MetadataType,
    query::ResponseWrapper,
    token::{Locks, SubModules},
};
use komple_utils::storage::StorageHelper;

pub fn token_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(crate::contract::reply);
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

const USER: &str = "juno.user";
const ADMIN: &str = "juno.admin";
const RANDOM: &str = "juno.random";
const RANDOM_2: &str = "juno.random2";
const NATIVE_DENOM: &str = "denom";
const TEST_DENOM: &str = "test_denom";

fn mock_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(ADMIN),
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
                    denom: TEST_DENOM.to_string(),
                    amount: Uint128::new(1_000_000),
                }],
            )
            .unwrap();
    })
}

fn proper_instantiate(
    app: &mut App,
    minter: String,
    per_address_limit: Option<u32>,
    start_time: Option<Timestamp>,
    max_token_limit: Option<u32>,
    ipfs_link: Option<String>,
) -> Addr {
    let token_code_id = app.store_code(token_module());
    let metadata_code_id = app.store_code(metadata_module());

    let token_info = TokenInfo {
        symbol: "TTT".to_string(),
        minter,
    };
    let collection_config = CollectionConfig {
        per_address_limit,
        start_time,
        max_token_limit,
        ipfs_link,
    };
    let metadata_info = MetadataInfo {
        instantiate_msg: MetadataInstantiateMsg {
            metadata_type: MetadataType::Standard,
        },
        code_id: metadata_code_id,
    };
    let msg = InstantiateMsg {
        creator: ADMIN.to_string(),
        token_info,
        collection_type: Collections::Standard,
        collection_name: "Test Collection".to_string(),
        collection_config,
        metadata_info,
    };
    let register_msg = RegisterMsg {
        admin: ADMIN.to_string(),
        data: Some(to_binary(&msg).unwrap()),
    };

    app.instantiate_contract(
        token_code_id,
        Addr::unchecked(ADMIN),
        &register_msg,
        &[],
        "test",
        None,
    )
    .unwrap()
}

mod initialization {
    use super::*;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let token_code_id = app.store_code(token_module());
        let metadata_code_id = app.store_code(metadata_module());

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
        let msg = InstantiateMsg {
            creator: ADMIN.to_string(),
            token_info,
            collection_type: Collections::Standard,
            collection_name: "Test Collection".to_string(),
            collection_config,
            metadata_info,
        };
        let register_msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(to_binary(&msg).unwrap()),
        };
        let token_module_addr = app
            .instantiate_contract(
                token_code_id,
                Addr::unchecked(ADMIN),
                &register_msg,
                &[],
                "test",
                None,
            )
            .unwrap();
        assert_eq!(token_module_addr, "contract0");
    }

    #[test]
    fn test_invalid_time() {
        let mut app = mock_app();
        let token_code_id = app.store_code(token_module());
        let metadata_code_id = app.store_code(metadata_module());

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
        let msg = InstantiateMsg {
            creator: ADMIN.to_string(),
            token_info: token_info.clone(),
            collection_type: Collections::Standard,
            collection_name: "Test Collection".to_string(),
            collection_config: collection_config.clone(),
            metadata_info: metadata_info.clone(),
        };
        let register_msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(to_binary(&msg).unwrap()),
        };
        let err = app
            .instantiate_contract(
                token_code_id,
                Addr::unchecked(ADMIN),
                &register_msg,
                &[],
                "test",
                None,
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            ContractError::InvalidStartTime {}.to_string()
        );

        collection_config.start_time = Some(app.block_info().time.minus_seconds(10));
        let msg = InstantiateMsg {
            creator: ADMIN.to_string(),
            token_info,
            collection_type: Collections::Standard,
            collection_name: "Test Collection".to_string(),
            collection_config,
            metadata_info,
        };
        let register_msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(to_binary(&msg).unwrap()),
        };
        let err = app
            .instantiate_contract(
                token_code_id,
                Addr::unchecked(ADMIN),
                &register_msg,
                &[],
                "test",
                None,
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            ContractError::InvalidStartTime {}.to_string()
        );
    }

    #[test]
    fn test_invalid_max_token_limit() {
        let mut app = mock_app();
        let token_code_id = app.store_code(token_module());
        let metadata_code_id = app.store_code(metadata_module());

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
        let msg = InstantiateMsg {
            creator: ADMIN.to_string(),
            token_info,
            collection_type: Collections::Standard,
            collection_name: "Test Collection".to_string(),
            collection_config,
            metadata_info,
        };
        let register_msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(to_binary(&msg).unwrap()),
        };
        let err = app
            .instantiate_contract(
                token_code_id,
                Addr::unchecked(ADMIN),
                &register_msg,
                &[],
                "test",
                None,
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            ContractError::InvalidMaxTokenLimit {}.to_string()
        );
    }

    #[test]
    fn test_invalid_per_address_limit() {
        let mut app = mock_app();
        let token_code_id = app.store_code(token_module());
        let metadata_code_id = app.store_code(metadata_module());

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
        let msg = InstantiateMsg {
            creator: ADMIN.to_string(),
            token_info,
            collection_type: Collections::Standard,
            collection_name: "Test Collection".to_string(),
            collection_config,
            metadata_info,
        };
        let register_msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(to_binary(&msg).unwrap()),
        };
        let err = app
            .instantiate_contract(
                token_code_id,
                Addr::unchecked(ADMIN),
                &register_msg,
                &[],
                "test",
                None,
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            ContractError::InvalidPerAddressLimit {}.to_string()
        );
    }

    #[test]
    fn test_missing_ipfs_link() {
        let mut app = mock_app();
        let token_code_id = app.store_code(token_module());
        let metadata_code_id = app.store_code(metadata_module());

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
        let msg = InstantiateMsg {
            creator: ADMIN.to_string(),
            token_info,
            collection_type: Collections::Standard,
            collection_name: "Test Collection".to_string(),
            collection_config,
            metadata_info,
        };
        let register_msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(to_binary(&msg).unwrap()),
        };
        let err = app
            .instantiate_contract(
                token_code_id,
                Addr::unchecked(ADMIN),
                &register_msg,
                &[],
                "test",
                None,
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            ContractError::IpfsNotFound {}.to_string()
        );
    }

    #[test]
    fn test_invalid_collection_metadata_type() {
        let mut app = mock_app();
        let token_code_id = app.store_code(token_module());
        let metadata_code_id = app.store_code(metadata_module());

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
                metadata_type: MetadataType::Dynamic,
            },
            code_id: metadata_code_id,
        };
        let mut msg = InstantiateMsg {
            creator: ADMIN.to_string(),
            token_info,
            collection_type: Collections::Standard,
            collection_name: "Test Collection".to_string(),
            collection_config,
            metadata_info: metadata_info.clone(),
        };
        let register_msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(to_binary(&msg).unwrap()),
        };

        let err = app
            .instantiate_contract(
                token_code_id,
                Addr::unchecked(ADMIN),
                &register_msg,
                &[],
                "test",
                None,
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            ContractError::InvalidCollectionMetadataType {}.to_string()
        );

        msg.metadata_info.instantiate_msg.metadata_type = MetadataType::Standard;
        msg.collection_type = Collections::Linked;
        let err = app
            .instantiate_contract(
                token_code_id,
                Addr::unchecked(ADMIN),
                &register_msg,
                &[],
                "test",
                None,
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
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
            let token_module_addr = proper_instantiate(
                &mut app,
                ADMIN.to_string(),
                None,
                None,
                None,
                Some("some-link".to_string()),
            );

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
            let token_module_addr = proper_instantiate(
                &mut app,
                ADMIN.to_string(),
                None,
                None,
                None,
                Some("some-link".to_string()),
            );

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
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    None,
                    None,
                    Some("some-link".to_string()),
                );

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
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    None,
                    None,
                    Some("some-link".to_string()),
                );

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
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    None,
                    None,
                    Some("some-link".to_string()),
                );

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::Mint {
                        owner: USER.to_string(),
                        metadata_id: None,
                    },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
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
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    None,
                    None,
                    Some("some-link".to_string()),
                );

                let locks = Locks {
                    mint_lock: false,
                    burn_lock: true,
                    transfer_lock: true,
                    send_lock: false,
                };
                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::UpdateTokenLocks {
                        token_id: "1".to_string(),
                        locks: locks,
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
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    None,
                    None,
                    Some("some-link".to_string()),
                );

                let locks = Locks {
                    mint_lock: false,
                    burn_lock: true,
                    transfer_lock: true,
                    send_lock: false,
                };
                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::UpdateTokenLocks {
                        token_id: "1".to_string(),
                        locks: locks,
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

        mod update_per_address_limit {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    None,
                    None,
                    Some("some-link".to_string()),
                );

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::UpdatePerAddressLimit {
                        per_address_limit: Some(5),
                    },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
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
            fn test_invalid_admin() {
                let mut app = mock_app();
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    None,
                    None,
                    Some("some-link".to_string()),
                );

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::UpdatePerAddressLimit {
                        per_address_limit: Some(5),
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
            fn test_invalid_per_address_limit() {
                let mut app = mock_app();
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    None,
                    None,
                    Some("some-link".to_string()),
                );

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::UpdatePerAddressLimit {
                        per_address_limit: Some(0),
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

        mod update_start_time {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    None,
                    None,
                    Some("some-link".to_string()),
                );

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::UpdateStartTime {
                        start_time: Some(app.block_info().time.plus_seconds(5)),
                    },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
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
            fn test_invalid_admin() {
                let mut app = mock_app();
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    None,
                    None,
                    Some("some-link".to_string()),
                );

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::UpdateStartTime {
                        start_time: Some(app.block_info().time.plus_seconds(5)),
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
            fn test_invalid_time() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(5);
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    Some(start_time),
                    None,
                    Some("some-link".to_string()),
                );

                let genesis_time = app.block_info().time;

                app.update_block(|block| block.time = block.time.plus_seconds(10));

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::UpdateStartTime {
                        start_time: Some(app.block_info().time.plus_seconds(5)),
                    },
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::AlreadyStarted {}.to_string()
                );

                app.update_block(|block| block.time = block.time.minus_seconds(6));

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::UpdateStartTime {
                        start_time: Some(genesis_time.plus_seconds(2)),
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
    }

    mod operations {
        use super::*;

        mod transfer_operation {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    None,
                    None,
                    Some("some-link".to_string()),
                );

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::Mint {
                        owner: USER.to_string(),
                        metadata_id: None,
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
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    None,
                    None,
                    Some("some-link".to_string()),
                );

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::Mint {
                        owner: USER.to_string(),
                        metadata_id: None,
                    },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
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
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    None,
                    None,
                    Some("some-link".to_string()),
                );

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::Mint {
                        owner: USER.to_string(),
                        metadata_id: None,
                    },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
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
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    None,
                    None,
                    Some("some-link".to_string()),
                );

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::Mint {
                        owner: USER.to_string(),
                        metadata_id: None,
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

        mod burn_operation {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    None,
                    None,
                    Some("some-link".to_string()),
                );

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::Mint {
                        owner: USER.to_string(),
                        metadata_id: None,
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
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    None,
                    None,
                    Some("some-link".to_string()),
                );

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::Mint {
                        owner: USER.to_string(),
                        metadata_id: None,
                    },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
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
            use komple_metadata_module::msg::MetadataResponse;

            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    None,
                    None,
                    Some("some-link".to_string()),
                );

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::Mint {
                        owner: USER.to_string(),
                        metadata_id: None,
                    },
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_module_addr.clone(),
                        &msg,
                        &[coin(1_000_000, NATIVE_DENOM)],
                    )
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
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    None,
                    None,
                    Some("some-link".to_string()),
                );

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

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::Mint {
                        owner: USER.to_string(),
                        metadata_id: None,
                    },
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::MintLocked {}.to_string()
                );
            }

            #[test]
            fn test_max_token_limit() {
                let mut app = mock_app();
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    None,
                    Some(2),
                    Some("some-link".to_string()),
                );

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::Mint {
                        owner: USER.to_string(),
                        metadata_id: None,
                    },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
                    .unwrap();

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::Mint {
                        owner: RANDOM.to_string(),
                        metadata_id: None,
                    },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
                    .unwrap();

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::Mint {
                        owner: RANDOM_2.to_string(),
                        metadata_id: None,
                    },
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::TokenLimitReached {}.to_string()
                );
            }

            #[test]
            fn test_per_address_limit() {
                let mut app = mock_app();
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    Some(2),
                    None,
                    None,
                    Some("some-link".to_string()),
                );

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::Mint {
                        owner: USER.to_string(),
                        metadata_id: None,
                    },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
                    .unwrap();

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::Mint {
                        owner: USER.to_string(),
                        metadata_id: None,
                    },
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
                    .unwrap();

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::Mint {
                        owner: USER.to_string(),
                        metadata_id: None,
                    },
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::TokenLimitReached {}.to_string()
                );
            }

            #[test]
            fn test_invalid_time() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(5);
                let token_module_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    Some(start_time),
                    None,
                    Some("some-link".to_string()),
                );

                let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                    msg: ExecuteMsg::Mint {
                        owner: USER.to_string(),
                        metadata_id: None,
                    },
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), token_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::MintingNotStarted {}.to_string()
                );
            }
        }
    }
}
