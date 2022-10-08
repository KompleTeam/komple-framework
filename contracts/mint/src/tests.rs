use crate::msg::QueryMsg;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use cosmwasm_std::{Addr, Coin, Decimal, Empty, Uint128};
use cw721_base::msg::{ExecuteMsg as Cw721ExecuteMsg, QueryMsg as Cw721QueryMsg};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_token_module::state::CollectionConfig;
use komple_token_module::{
    msg::{
        ExecuteMsg as TokenExecuteMsg, InstantiateMsg as TokenInstantiateMsg,
        QueryMsg as TokenQueryMsg, TokenInfo,
    },
    state::{CollectionInfo, Contracts},
};
use komple_types::{
    collection::Collections, metadata::Metadata as MetadataType, query::ResponseWrapper,
};
use komple_utils::query_collection_address;

pub fn minter_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(crate::contract::reply);
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

pub fn metadata_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_metadata_module::contract::execute,
        komple_metadata_module::contract::instantiate,
        komple_metadata_module::contract::query,
    );
    Box::new(contract)
}

const USER: &str = "juno1shfqtuup76mngspx29gcquykjvvlx9na4kymlm";
const ADMIN: &str = "juno1qamfln8u5w8d3vlhp5t9mhmylfkgad4jz6t7cv";
// const RANDOM: &str = "juno1et88c8yd6xr8azkmp02lxtctkqq36lt63tdt7e";
const NATIVE_DENOM: &str = "denom";

fn mock_app() -> App {
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
    })
}

fn proper_instantiate(app: &mut App) -> Addr {
    let minter_code_id = app.store_code(minter_contract());

    let msg = InstantiateMsg {
        admin: ADMIN.to_string(),
    };
    let minter_contract_addr = app
        .instantiate_contract(
            minter_code_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "test",
            None,
        )
        .unwrap();

    minter_contract_addr
}

fn setup_collection(
    app: &mut App,
    minter_addr: &Addr,
    sender: Addr,
    linked_collections: Option<Vec<u32>>,
    unit_price: Option<Uint128>,
) {
    let token_code_id = app.store_code(token_module());

    let collection_info = CollectionInfo {
        collection_type: Collections::Standard,
        name: "Test Collection".to_string(),
        description: "Test Description".to_string(),
        image: "ipfs://xyz".to_string(),
        external_link: None,
    };
    let token_info = TokenInfo {
        symbol: "TEST".to_string(),
        minter: minter_addr.to_string(),
    };
    let collection_config = CollectionConfig {
        per_address_limit: None,
        start_time: None,
        unit_price,
        max_token_limit: None,
        native_denom: NATIVE_DENOM.to_string(),
        ipfs_link: Some("some-link".to_string()),
    };

    let msg = ExecuteMsg::CreateCollection {
        code_id: token_code_id,
        token_instantiate_msg: TokenInstantiateMsg {
            admin: ADMIN.to_string(),
            creator: ADMIN.to_string(),
            collection_info,
            collection_config,
            token_info,
            royalty_share: Some(Decimal::new(Uint128::new(5))),
        },
        linked_collections,
    };
    let _ = app
        .execute_contract(sender, minter_addr.clone(), &msg, &vec![])
        .unwrap();
}

fn setup_metadata_module(
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

mod actions {
    use super::*;

    mod mint {
        use super::*;
        use crate::{
            msg::{ExecuteMsg, QueryMsg},
            ContractError,
        };
        use cosmwasm_std::coin;
        use cw721::OwnerOfResponse;
        use komple_token_module::msg::QueryMsg as TokenQueryMsg;
        use komple_types::query::ResponseWrapper;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);
            setup_collection(
                &mut app,
                &minter_addr,
                Addr::unchecked(ADMIN),
                None,
                Some(Uint128::new(50_000)),
            );

            let collection_addr = query_collection_address(&app.wrap(), &minter_addr, &1).unwrap();
            setup_metadata_module(&mut app, collection_addr, MetadataType::Standard);

            let res = app.wrap().query_balance(ADMIN, NATIVE_DENOM).unwrap();
            assert_eq!(res.amount, Uint128::new(0));

            let msg = ExecuteMsg::Mint {
                collection_id: 1,
                metadata_id: None,
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(USER),
                    minter_addr.clone(),
                    &msg,
                    &vec![coin(50_000, NATIVE_DENOM)],
                )
                .unwrap();

            let msg = QueryMsg::CollectionAddress(1);
            let response: ResponseWrapper<String> =
                app.wrap().query_wasm_smart(minter_addr, &msg).unwrap();
            let token_address = response.data;

            let msg: Cw721QueryMsg<TokenQueryMsg> = Cw721QueryMsg::OwnerOf {
                token_id: "1".to_string(),
                include_expired: None,
            };
            let response: OwnerOfResponse =
                app.wrap().query_wasm_smart(token_address, &msg).unwrap();
            assert_eq!(response.owner, USER.to_string());

            let res = app.wrap().query_balance(ADMIN, NATIVE_DENOM).unwrap();
            assert_eq!(res.amount, Uint128::new(50_000));
        }

        #[test]
        fn test_locked_minting() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);

            let collection_addr = query_collection_address(&app.wrap(), &minter_addr, &1).unwrap();
            setup_metadata_module(&mut app, collection_addr, MetadataType::Standard);

            let msg = ExecuteMsg::UpdateMintLock { lock: true };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
                .unwrap();

            let msg = ExecuteMsg::Mint {
                collection_id: 1,
                metadata_id: None,
            };
            let err = app
                .execute_contract(Addr::unchecked(USER), minter_addr, &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::LockedMint {}.to_string()
            )
        }
    }

    mod locks {
        use komple_types::query::ResponseWrapper;

        use super::*;
        use crate::{
            msg::{ExecuteMsg, QueryMsg},
            state::Config,
        };

        #[test]
        fn test_mint_lock_happy_path() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateMintLock { lock: true };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
                .unwrap();

            let msg = QueryMsg::Config {};
            let response: ResponseWrapper<Config> =
                app.wrap().query_wasm_smart(minter_addr, &msg).unwrap();
            assert_eq!(response.data.mint_lock, true);
        }
    }

    mod collections {
        use komple_types::query::ResponseWrapper;

        use super::*;

        use crate::{
            msg::{CollectionsResponse, ExecuteMsg, QueryMsg},
            ContractError,
        };

        mod creation {
            use super::*;

            #[test]
            fn test_collection_creation() {
                let mut app = mock_app();
                let minter_addr = proper_instantiate(&mut app);
                let token_code_id = app.store_code(token_module());

                let collection_info = CollectionInfo {
                    collection_type: Collections::Standard,
                    name: "Test Collection".to_string(),
                    description: "Test Description".to_string(),
                    image: "ipfs://xyz".to_string(),
                    external_link: None,
                };
                let token_info = TokenInfo {
                    symbol: "TEST".to_string(),
                    minter: minter_addr.to_string(),
                };
                let collection_config = CollectionConfig {
                    per_address_limit: None,
                    start_time: None,
                    unit_price: None,
                    native_denom: NATIVE_DENOM.to_string(),
                    max_token_limit: None,
                    ipfs_link: Some("some-link".to_string()),
                };
                let msg = ExecuteMsg::CreateCollection {
                    code_id: token_code_id,
                    token_instantiate_msg: TokenInstantiateMsg {
                        admin: ADMIN.to_string(),
                        creator: ADMIN.to_string(),
                        collection_info,
                        collection_config,
                        token_info,
                        royalty_share: Some(Decimal::new(Uint128::new(5))),
                    },
                    linked_collections: None,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
                    .unwrap();

                let msg = QueryMsg::CollectionAddress(1);
                let res: ResponseWrapper<String> =
                    app.wrap().query_wasm_smart(minter_addr, &msg).unwrap();
                assert_eq!(res.data, "contract1");
            }

            #[test]
            fn test_invalid_admin() {
                let mut app = mock_app();
                let minter_addr = proper_instantiate(&mut app);
                let token_code_id = app.store_code(token_module());

                let collection_info = CollectionInfo {
                    collection_type: Collections::Standard,
                    name: "Test Collection".to_string(),
                    description: "Test Description".to_string(),
                    image: "ipfs://xyz".to_string(),
                    external_link: None,
                };
                let token_info = TokenInfo {
                    symbol: "TEST".to_string(),
                    minter: minter_addr.to_string(),
                };
                let collection_config = CollectionConfig {
                    per_address_limit: None,
                    start_time: None,
                    unit_price: None,
                    native_denom: NATIVE_DENOM.to_string(),
                    max_token_limit: None,
                    ipfs_link: None,
                };
                let msg = ExecuteMsg::CreateCollection {
                    code_id: token_code_id,
                    token_instantiate_msg: TokenInstantiateMsg {
                        admin: ADMIN.to_string(),
                        creator: ADMIN.to_string(),
                        collection_info,
                        collection_config,
                        token_info,
                        royalty_share: Some(Decimal::new(Uint128::new(5))),
                    },
                    linked_collections: None,
                };
                let err = app
                    .execute_contract(Addr::unchecked(USER), minter_addr.clone(), &msg, &vec![])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::Unauthorized {}.to_string()
                );
            }

            #[test]
            fn test_public_happy_path() {
                let mut app = mock_app();
                let minter_addr = proper_instantiate(&mut app);

                let msg = ExecuteMsg::UpdatePublicCollectionCreation {
                    public_collection_creation: true,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
                    .unwrap();

                setup_collection(&mut app, &minter_addr, Addr::unchecked(USER), None, None);

                let msg = QueryMsg::CollectionAddress(1);
                let res: ResponseWrapper<String> =
                    app.wrap().query_wasm_smart(minter_addr, &msg).unwrap();
                assert_eq!(res.data, "contract1");
            }
        }

        mod update_public_collection_creation {
            use super::*;

            #[test]
            fn test_invalid_owner() {
                let mut app = mock_app();
                let minter_addr = proper_instantiate(&mut app);

                let msg = ExecuteMsg::UpdatePublicCollectionCreation {
                    public_collection_creation: true,
                };
                let err = app
                    .execute_contract(Addr::unchecked(USER), minter_addr.clone(), &msg, &vec![])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::Unauthorized {}.to_string()
                )
            }
        }

        #[test]
        fn test_linked_collections_happy_path() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);

            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);
            setup_collection(
                &mut app,
                &minter_addr,
                Addr::unchecked(ADMIN),
                Some(vec![1]),
                None,
            );
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);

            let msg = ExecuteMsg::UpdateLinkedCollections {
                collection_id: 4,
                linked_collections: vec![1, 3],
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
                .unwrap();

            let msg = QueryMsg::LinkedCollections { collection_id: 2 };
            let res: ResponseWrapper<Vec<u32>> = app
                .wrap()
                .query_wasm_smart(minter_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data, vec![1]);

            let msg = QueryMsg::LinkedCollections { collection_id: 4 };
            let res: ResponseWrapper<Vec<u32>> =
                app.wrap().query_wasm_smart(minter_addr, &msg).unwrap();
            assert_eq!(res.data, vec![1, 3]);
        }

        #[test]
        fn test_linked_collections_unhappy_path() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);

            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);

            let msg = ExecuteMsg::UpdateLinkedCollections {
                collection_id: 5,
                linked_collections: vec![10],
            };
            let err = app
                .execute_contract(Addr::unchecked(USER), minter_addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );

            let err = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::CollectionIdNotFound {}.to_string()
            );

            let msg = ExecuteMsg::UpdateLinkedCollections {
                collection_id: 2,
                linked_collections: vec![2],
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::SelfLinkedCollection {}.to_string()
            );

            let msg = ExecuteMsg::UpdateLinkedCollections {
                collection_id: 2,
                linked_collections: vec![10],
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr, &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::CollectionIdNotFound {}.to_string()
            );
        }

        #[test]
        fn test_collections_query() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);

            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);

            let msg = QueryMsg::Collections {
                blacklist: false,
                start_after: None,
                limit: None,
            };
            let res: ResponseWrapper<Vec<CollectionsResponse>> = app
                .wrap()
                .query_wasm_smart(minter_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.len(), 7);
            assert_eq!(res.data[3].collection_id, 4);
            assert_eq!(res.data[3].address, "contract4");

            let msg = QueryMsg::Collections {
                blacklist: false,
                start_after: Some(2),
                limit: Some(4),
            };
            let res: ResponseWrapper<Vec<CollectionsResponse>> = app
                .wrap()
                .query_wasm_smart(minter_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.len(), 4);
            assert_eq!(res.data[3].collection_id, 6);
            assert_eq!(res.data[3].address, "contract6");
        }
    }

    mod blacklist {
        use crate::{msg::CollectionsResponse, ContractError};

        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);
            let collection_addr = query_collection_address(&app.wrap(), &minter_addr, &1).unwrap();
            setup_metadata_module(&mut app, collection_addr.clone(), MetadataType::Standard);

            let msg = ExecuteMsg::BlacklistCollection { collection_id: 1 };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
                .unwrap();

            let res = query_collection_address(&app.wrap(), &minter_addr, &1).unwrap_err();
            assert_eq!(res.to_string(), "Collection not found");

            let msg = QueryMsg::Collections {
                blacklist: true,
                start_after: None,
                limit: None,
            };
            let res: ResponseWrapper<Vec<CollectionsResponse>> = app
                .wrap()
                .query_wasm_smart(minter_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.len(), 1);
            assert_eq!(
                res.data[0],
                CollectionsResponse {
                    collection_id: 1,
                    address: collection_addr.to_string()
                }
            );
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);
            let collection_addr = query_collection_address(&app.wrap(), &minter_addr, &1).unwrap();
            setup_metadata_module(&mut app, collection_addr.clone(), MetadataType::Standard);

            let msg = ExecuteMsg::BlacklistCollection { collection_id: 1 };
            let err = app
                .execute_contract(Addr::unchecked(USER), minter_addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }

        #[test]
        fn test_existing_blacklist() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);
            let collection_addr = query_collection_address(&app.wrap(), &minter_addr, &1).unwrap();
            setup_metadata_module(&mut app, collection_addr.clone(), MetadataType::Standard);

            let msg = ExecuteMsg::BlacklistCollection { collection_id: 1 };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
                .unwrap();
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AlreadyBlacklisted {}.to_string()
            );
        }

        #[test]
        fn test_absent_collection() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);
            let collection_addr = query_collection_address(&app.wrap(), &minter_addr, &1).unwrap();
            setup_metadata_module(&mut app, collection_addr.clone(), MetadataType::Standard);

            let msg = ExecuteMsg::BlacklistCollection { collection_id: 2 };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::CollectionIdNotFound {}.to_string()
            );
        }
    }

    mod whitelist {
        use crate::ContractError;

        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);
            let collection_addr = query_collection_address(&app.wrap(), &minter_addr, &1).unwrap();
            setup_metadata_module(&mut app, collection_addr.clone(), MetadataType::Standard);

            let msg = ExecuteMsg::BlacklistCollection { collection_id: 1 };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
                .unwrap();

            let msg = ExecuteMsg::WhitelistCollection { collection_id: 1 };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
                .unwrap();

            let _ = query_collection_address(&app.wrap(), &minter_addr, &1).unwrap();
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);
            let collection_addr = query_collection_address(&app.wrap(), &minter_addr, &1).unwrap();
            setup_metadata_module(&mut app, collection_addr.clone(), MetadataType::Standard);

            let msg = ExecuteMsg::WhitelistCollection { collection_id: 1 };
            let err = app
                .execute_contract(Addr::unchecked(USER), minter_addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }

        #[test]
        fn test_existing_whitelist() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);
            let collection_addr = query_collection_address(&app.wrap(), &minter_addr, &1).unwrap();
            setup_metadata_module(&mut app, collection_addr.clone(), MetadataType::Standard);

            let msg = ExecuteMsg::WhitelistCollection { collection_id: 1 };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AlreadyWhitelistlisted {}.to_string()
            );
        }

        #[test]
        fn test_absent_collection() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None, None);
            let collection_addr = query_collection_address(&app.wrap(), &minter_addr, &1).unwrap();
            setup_metadata_module(&mut app, collection_addr.clone(), MetadataType::Standard);

            let msg = ExecuteMsg::WhitelistCollection { collection_id: 2 };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::CollectionIdNotFound {}.to_string()
            );
        }
    }
}
