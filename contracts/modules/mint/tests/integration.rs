use cosmwasm_std::{Addr, Coin, Empty, Uint128};
use cw721_base::msg::QueryMsg as Cw721QueryMsg;
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_framework_metadata_module::msg::InstantiateMsg as MetadataInstantiateMsg;
use komple_framework_mint_module::msg::{
    CollectionFundInfo, CollectionsResponse, ExecuteMsg, QueryMsg,
};
use komple_framework_mint_module::state::CollectionInfo;
use komple_framework_mint_module::ContractError;
use komple_framework_token_module::{
    msg::{MetadataInfo, TokenInfo},
    state::CollectionConfig,
};
use komple_types::modules::metadata::Metadata as MetadataType;
use komple_types::modules::mint::Collections;
use komple_types::shared::query::ResponseWrapper;
use komple_types::shared::RegisterMsg;
use komple_utils::funds::FundsError;
use komple_utils::storage::StorageHelper;

pub fn minter_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_framework_mint_module::contract::execute,
        komple_framework_mint_module::contract::instantiate,
        komple_framework_mint_module::contract::query,
    )
    .with_reply(komple_framework_mint_module::contract::reply);
    Box::new(contract)
}

pub fn token_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_framework_token_module::contract::execute,
        komple_framework_token_module::contract::instantiate,
        komple_framework_token_module::contract::query,
    )
    .with_reply(komple_framework_token_module::contract::reply);
    Box::new(contract)
}

pub fn metadata_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_framework_metadata_module::contract::execute,
        komple_framework_metadata_module::contract::instantiate,
        komple_framework_metadata_module::contract::query,
    );
    Box::new(contract)
}

pub fn cw20_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

const USER: &str = "juno..user";
const ADMIN: &str = "juno..admin";
// const RANDOM: &str = "juno1et88c8yd6xr8azkmp02lxtctkqq36lt63tdt7e";
const NATIVE_DENOM: &str = "denom";
const CW20_DENOM: &str = "cwdenom";

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

    let msg = RegisterMsg {
        admin: ADMIN.to_string(),
        data: None,
    };

    app.instantiate_contract(
        minter_code_id,
        Addr::unchecked(ADMIN),
        &msg,
        &[],
        "test",
        Some(ADMIN.to_string()),
    )
    .unwrap()
}

fn setup_collection(
    app: &mut App,
    minter_addr: &Addr,
    sender: Addr,
    linked_collections: Option<Vec<u32>>,
) {
    let token_code_id = app.store_code(token_module());
    let metadata_code_id = app.store_code(metadata_module());

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
        max_token_limit: None,
        ipfs_link: Some("some-link".to_string()),
    };
    let metadata_info = MetadataInfo {
        instantiate_msg: MetadataInstantiateMsg {
            metadata_type: MetadataType::Standard,
        },
        code_id: metadata_code_id,
    };
    let fund_info = CollectionFundInfo {
        is_native: true,
        denom: NATIVE_DENOM.to_string(),
        cw20_address: None,
    };
    let msg = ExecuteMsg::CreateCollection {
        code_id: token_code_id,
        collection_config,
        collection_info,
        metadata_info,
        token_info,
        fund_info,
        linked_collections,
    };
    let _ = app
        .execute_contract(sender, minter_addr.clone(), &msg, &[])
        .unwrap();
}

fn setup_cw20_token(app: &mut App) -> Addr {
    let cw20_code_id = app.store_code(cw20_contract());
    let msg = cw20_base::msg::InstantiateMsg {
        name: "Test Token".to_string(),
        symbol: CW20_DENOM.to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: None,
        marketing: None,
    };
    app.instantiate_contract(
        cw20_code_id,
        Addr::unchecked(ADMIN),
        &msg,
        &[],
        "test",
        Some(ADMIN.to_string()),
    )
    .unwrap()
}

mod actions {
    use super::*;

    mod mint {
        use super::*;
        use cw721::OwnerOfResponse;
        use komple_framework_token_module::msg::QueryMsg as TokenQueryMsg;
        use komple_types::shared::query::ResponseWrapper;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);

            let msg = ExecuteMsg::Mint {
                collection_id: 1,
                metadata_id: None,
            };
            let _ = app
                .execute_contract(Addr::unchecked(USER), minter_addr.clone(), &msg, &[])
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
        }

        #[test]
        fn test_locked_minting() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);

            let msg = ExecuteMsg::UpdateCollectionMintLock {
                collection_id: 1,
                lock: true,
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &[])
                .unwrap();

            let msg = ExecuteMsg::Mint {
                collection_id: 1,
                metadata_id: None,
            };
            let err = app
                .execute_contract(Addr::unchecked(USER), minter_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::LockedMint {}.to_string()
            )
        }
    }

    mod locks {
        use super::*;

        #[test]
        fn test_mint_lock_happy_path() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateCollectionMintLock {
                collection_id: 1,
                lock: true,
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &[])
                .unwrap();

            let msg = QueryMsg::MintLock { collection_id: 1 };
            let response: ResponseWrapper<Option<bool>> = app
                .wrap()
                .query_wasm_smart(minter_addr.clone(), &msg)
                .unwrap();
            assert_eq!(response.data, Some(true));

            let msg = QueryMsg::MintLock { collection_id: 2 };
            let response: ResponseWrapper<Option<bool>> =
                app.wrap().query_wasm_smart(minter_addr, &msg).unwrap();
            assert_eq!(response.data, None);
        }
    }

    mod collections {
        use super::*;

        mod creation {
            use super::*;

            #[test]
            fn test_collection_creation() {
                let mut app = mock_app();
                let minter_addr = proper_instantiate(&mut app);
                let token_code_id = app.store_code(token_module());
                let metadata_code_id = app.store_code(metadata_module());

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
                    max_token_limit: None,
                    ipfs_link: Some("some-link".to_string()),
                };
                let metadata_info = MetadataInfo {
                    instantiate_msg: MetadataInstantiateMsg {
                        metadata_type: MetadataType::Standard,
                    },
                    code_id: metadata_code_id,
                };
                let fund_info = CollectionFundInfo {
                    is_native: true,
                    denom: NATIVE_DENOM.to_string(),
                    cw20_address: None,
                };
                let create_collection_msg = ExecuteMsg::CreateCollection {
                    code_id: token_code_id,
                    collection_config: collection_config.clone(),
                    collection_info: collection_info.clone(),
                    metadata_info: metadata_info.clone(),
                    token_info: token_info.clone(),
                    fund_info,
                    linked_collections: None,
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        minter_addr.clone(),
                        &create_collection_msg,
                        &[],
                    )
                    .unwrap();

                let msg = QueryMsg::CollectionAddress(1);
                let res: ResponseWrapper<String> = app
                    .wrap()
                    .query_wasm_smart(minter_addr.clone(), &msg)
                    .unwrap();
                assert_eq!(res.data, "contract1");

                let msg = QueryMsg::CollectionInfo { collection_id: 1 };
                let res: ResponseWrapper<CollectionInfo> = app
                    .wrap()
                    .query_wasm_smart(minter_addr.clone(), &msg)
                    .unwrap();
                assert_eq!(res.data.name, "Test Collection");
                assert_eq!(res.data.collection_type, Collections::Standard);

                let res = app.wrap().query_wasm_contract_info("contract1").unwrap();
                assert_eq!(res.admin, Some(ADMIN.to_string()));

                let cw20_addr = setup_cw20_token(&mut app);
                let fund_info = CollectionFundInfo {
                    is_native: false,
                    denom: CW20_DENOM.to_string(),
                    cw20_address: Some(cw20_addr.to_string()),
                };
                let create_collection_msg = ExecuteMsg::CreateCollection {
                    code_id: token_code_id,
                    collection_config,
                    collection_info,
                    metadata_info,
                    token_info,
                    fund_info,
                    linked_collections: None,
                };
                app.execute_contract(
                    Addr::unchecked(ADMIN),
                    minter_addr.clone(),
                    &create_collection_msg,
                    &[],
                )
                .unwrap();
            }

            #[test]
            fn test_invalid_admin() {
                let mut app = mock_app();
                let minter_addr = proper_instantiate(&mut app);
                let token_code_id = app.store_code(token_module());
                let metadata_code_id = app.store_code(metadata_module());

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
                    max_token_limit: None,
                    ipfs_link: None,
                };
                let metadata_info = MetadataInfo {
                    instantiate_msg: MetadataInstantiateMsg {
                        metadata_type: MetadataType::Standard,
                    },
                    code_id: metadata_code_id,
                };
                let fund_info = CollectionFundInfo {
                    is_native: true,
                    denom: NATIVE_DENOM.to_string(),
                    cw20_address: None,
                };
                let msg = ExecuteMsg::CreateCollection {
                    code_id: token_code_id,
                    collection_config,
                    collection_info,
                    metadata_info,
                    token_info,
                    fund_info,
                    linked_collections: None,
                };
                let err = app
                    .execute_contract(Addr::unchecked(USER), minter_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::Unauthorized {}.to_string()
                );
            }

            #[test]
            fn test_invalid_cw20_token() {
                let mut app = mock_app();
                let minter_addr = proper_instantiate(&mut app);
                let token_code_id = app.store_code(token_module());
                let metadata_code_id = app.store_code(metadata_module());
                let cw20_addr = setup_cw20_token(&mut app);

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
                    max_token_limit: None,
                    ipfs_link: Some("some-link".to_string()),
                };
                let metadata_info = MetadataInfo {
                    instantiate_msg: MetadataInstantiateMsg {
                        metadata_type: MetadataType::Standard,
                    },
                    code_id: metadata_code_id,
                };

                let fund_info = CollectionFundInfo {
                    is_native: false,
                    denom: "invalid".to_string(),
                    cw20_address: Some(cw20_addr.to_string()),
                };
                let create_collection_msg = ExecuteMsg::CreateCollection {
                    code_id: token_code_id,
                    collection_config: collection_config.clone(),
                    collection_info: collection_info.clone(),
                    metadata_info: metadata_info.clone(),
                    token_info: token_info.clone(),
                    fund_info,
                    linked_collections: None,
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        minter_addr.clone(),
                        &create_collection_msg,
                        &[],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    FundsError::InvalidCw20Token {}.to_string()
                );

                let fund_info = CollectionFundInfo {
                    is_native: false,
                    denom: CW20_DENOM.to_string(),
                    cw20_address: None,
                };
                let create_collection_msg = ExecuteMsg::CreateCollection {
                    code_id: token_code_id,
                    collection_config: collection_config.clone(),
                    collection_info: collection_info.clone(),
                    metadata_info: metadata_info.clone(),
                    token_info: token_info.clone(),
                    fund_info,
                    linked_collections: None,
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        minter_addr.clone(),
                        &create_collection_msg,
                        &[],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    FundsError::InvalidCw20Token {}.to_string()
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
                    .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &[])
                    .unwrap();

                setup_collection(&mut app, &minter_addr, Addr::unchecked(USER), None);

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
                    .execute_contract(Addr::unchecked(USER), minter_addr, &msg, &[])
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

            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);
            setup_collection(
                &mut app,
                &minter_addr,
                Addr::unchecked(ADMIN),
                Some(vec![1]),
            );
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);

            let msg = ExecuteMsg::UpdateLinkedCollections {
                collection_id: 4,
                linked_collections: vec![1, 3],
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &[])
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

            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);

            let msg = ExecuteMsg::UpdateLinkedCollections {
                collection_id: 5,
                linked_collections: vec![10],
            };
            let err = app
                .execute_contract(Addr::unchecked(USER), minter_addr.clone(), &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );

            let err = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &[])
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
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &[])
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
                .execute_contract(Addr::unchecked(ADMIN), minter_addr, &msg, &[])
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

            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);

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
            assert_eq!(res.data[3].address, "contract7");

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
            assert_eq!(res.data[3].address, "contract11");
        }
    }

    mod update_collection_status {
        use super::*;

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);
            setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);

            let msg = ExecuteMsg::UpdateCollectionStatus {
                collection_id: 1,
                is_blacklist: true,
            };
            let err = app
                .execute_contract(Addr::unchecked(USER), minter_addr.clone(), &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }

        mod blacklist {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let minter_addr = proper_instantiate(&mut app);
                setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);
                let collection_addr =
                    StorageHelper::query_collection_address(&app.wrap(), &minter_addr, &1).unwrap();

                let msg = ExecuteMsg::UpdateCollectionStatus {
                    collection_id: 1,
                    is_blacklist: true,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &[])
                    .unwrap();

                let res = StorageHelper::query_collection_address(&app.wrap(), &minter_addr, &1)
                    .unwrap_err();
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
            fn test_existing_blacklist() {
                let mut app = mock_app();
                let minter_addr = proper_instantiate(&mut app);
                setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);

                let msg = ExecuteMsg::UpdateCollectionStatus {
                    collection_id: 1,
                    is_blacklist: true,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &[])
                    .unwrap();
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &[])
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
                setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);

                let msg = ExecuteMsg::UpdateCollectionStatus {
                    collection_id: 2,
                    is_blacklist: true,
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::CollectionIdNotFound {}.to_string()
                );
            }
        }

        mod whitelist {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let minter_addr = proper_instantiate(&mut app);
                setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);

                let msg = ExecuteMsg::UpdateCollectionStatus {
                    collection_id: 1,
                    is_blacklist: true,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &[])
                    .unwrap();

                let msg = ExecuteMsg::UpdateCollectionStatus {
                    collection_id: 1,
                    is_blacklist: false,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &[])
                    .unwrap();

                let _ =
                    StorageHelper::query_collection_address(&app.wrap(), &minter_addr, &1).unwrap();
            }

            #[test]
            fn test_existing_whitelist() {
                let mut app = mock_app();
                let minter_addr = proper_instantiate(&mut app);
                setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);

                let msg = ExecuteMsg::UpdateCollectionStatus {
                    collection_id: 1,
                    is_blacklist: false,
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &[])
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
                setup_collection(&mut app, &minter_addr, Addr::unchecked(ADMIN), None);

                let msg = ExecuteMsg::UpdateCollectionStatus {
                    collection_id: 2,
                    is_blacklist: false,
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::CollectionIdNotFound {}.to_string()
                );
            }
        }
    }

    mod update_operators {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let mint_module_addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateOperators {
                addrs: vec![
                    "juno..first".to_string(),
                    "juno..second".to_string(),
                    "juno..first".to_string(),
                ],
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), mint_module_addr.clone(), &msg, &[])
                .unwrap();

            let msg = QueryMsg::Operators {};
            let res: ResponseWrapper<Vec<String>> = app
                .wrap()
                .query_wasm_smart(mint_module_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.len(), 2);
            assert_eq!(res.data[0], "juno..first");
            assert_eq!(res.data[1], "juno..second");

            let msg = ExecuteMsg::UpdateOperators {
                addrs: vec!["juno..third".to_string()],
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked("juno..first"),
                    mint_module_addr.clone(),
                    &msg,
                    &[],
                )
                .unwrap();

            let msg = QueryMsg::Operators {};
            let res: ResponseWrapper<Vec<String>> =
                app.wrap().query_wasm_smart(mint_module_addr, &msg).unwrap();
            assert_eq!(res.data.len(), 1);
            assert_eq!(res.data[0], "juno..third");
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let mint_module_addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateOperators {
                addrs: vec!["juno..first".to_string(), "juno..second".to_string()],
            };
            let err = app
                .execute_contract(Addr::unchecked(USER), mint_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }

        #[test]
        fn test_invalid_operator() {
            let mut app = mock_app();
            let mint_module_addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateOperators {
                addrs: vec!["juno..first".to_string(), "juno..second".to_string()],
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), mint_module_addr.clone(), &msg, &[])
                .unwrap();

            let err = app
                .execute_contract(Addr::unchecked("juno..third"), mint_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }
    }

    mod update_creators {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let mint_module_addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateCreators {
                addrs: vec![
                    "juno..first".to_string(),
                    "juno..second".to_string(),
                    "juno..first".to_string(),
                ],
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), mint_module_addr.clone(), &msg, &[])
                .unwrap();

            let msg = QueryMsg::Creators {};
            let res: ResponseWrapper<Vec<String>> = app
                .wrap()
                .query_wasm_smart(mint_module_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.len(), 2);
            assert_eq!(res.data[0], "juno..first");
            assert_eq!(res.data[1], "juno..second");

            setup_collection(
                &mut app,
                &mint_module_addr,
                Addr::unchecked("juno..second"),
                None,
            );
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let mint_module_addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateCreators {
                addrs: vec!["juno..first".to_string(), "juno..second".to_string()],
            };
            let err = app
                .execute_contract(Addr::unchecked(USER), mint_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }
    }

    mod lock_execute {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::LockExecute {};
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &vec![])
                .unwrap();

            let err = app
                .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::ExecuteLocked {}.to_string()
            );
        }
    }
}
