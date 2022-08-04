use crate::{msg::ConfigResponse, ContractError};
use cosmwasm_std::{coin, Addr, Coin, Empty, Timestamp, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use rift_types::collection::Collections;
use rift_types::query::ResponseWrapper;
use rift_types::tokens::Locks;
use rift_utils::{query_token_operation_lock, query_token_owner, FundsError};

use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TokenInfo},
    state::CollectionInfo,
};

pub fn token_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
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
    unit_price: Option<Coin>,
) -> Addr {
    let token_code_id = app.store_code(token_contract());

    let collection_info = CollectionInfo {
        collection_type: Collections::Normal,
        name: "Test Collection".to_string(),
        description: "Test Description".to_string(),
        image: "https://some-image.com".to_string(),
        external_link: None,
    };
    let token_info = TokenInfo {
        symbol: "TTT".to_string(),
        minter,
    };
    let msg = InstantiateMsg {
        admin: ADMIN.to_string(),
        token_info,
        per_address_limit,
        start_time,
        collection_info,
        max_token_limit,
        unit_price,
        native_denom: NATIVE_DENOM.to_string(),
    };
    let token_contract_addr = app
        .instantiate_contract(
            token_code_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "test",
            None,
        )
        .unwrap();

    token_contract_addr
}

mod initialization {
    use super::*;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let token_code_id = app.store_code(token_contract());

        let collection_info = CollectionInfo {
            collection_type: Collections::Normal,
            name: "Test Collection".to_string(),
            description: "Test Description".to_string(),
            image: "https://some-image.com".to_string(),
            external_link: None,
        };
        let token_info = TokenInfo {
            symbol: "TTT".to_string(),
            minter: ADMIN.to_string(),
        };
        let msg = InstantiateMsg {
            admin: ADMIN.to_string(),
            token_info,
            per_address_limit: Some(5),
            start_time: Some(app.block_info().time.plus_seconds(1)),
            collection_info,
            max_token_limit: Some(100),
            unit_price: Some(coin(100, NATIVE_DENOM)),
            native_denom: NATIVE_DENOM.to_string(),
        };
        let token_contract_addr = app
            .instantiate_contract(
                token_code_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();
        assert_eq!(token_contract_addr, "contract0");
    }

    #[test]
    fn test_invalid_time() {
        let mut app = mock_app();
        let token_code_id = app.store_code(token_contract());

        let collection_info = CollectionInfo {
            collection_type: Collections::Normal,
            name: "Test Collection".to_string(),
            description: "Test Description".to_string(),
            image: "https://some-image.com".to_string(),
            external_link: None,
        };
        let token_info = TokenInfo {
            symbol: "TTT".to_string(),
            minter: ADMIN.to_string(),
        };

        let msg = InstantiateMsg {
            admin: ADMIN.to_string(),
            token_info: token_info.clone(),
            per_address_limit: Some(5),
            start_time: Some(app.block_info().time),
            collection_info: collection_info.clone(),
            max_token_limit: Some(100),
            unit_price: Some(coin(100, NATIVE_DENOM)),
            native_denom: NATIVE_DENOM.to_string(),
        };
        let err = app
            .instantiate_contract(
                token_code_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            ContractError::InvalidStartTime {}.to_string()
        );

        let msg = InstantiateMsg {
            admin: ADMIN.to_string(),
            token_info: token_info.clone(),
            per_address_limit: Some(5),
            start_time: Some(app.block_info().time.minus_seconds(10)),
            collection_info: collection_info.clone(),
            max_token_limit: Some(100),
            unit_price: Some(coin(100, NATIVE_DENOM)),
            native_denom: NATIVE_DENOM.to_string(),
        };
        let err = app
            .instantiate_contract(
                token_code_id,
                Addr::unchecked(ADMIN),
                &msg,
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
        let token_code_id = app.store_code(token_contract());

        let collection_info = CollectionInfo {
            collection_type: Collections::Normal,
            name: "Test Collection".to_string(),
            description: "Test Description".to_string(),
            image: "https://some-image.com".to_string(),
            external_link: None,
        };
        let token_info = TokenInfo {
            symbol: "TTT".to_string(),
            minter: ADMIN.to_string(),
        };

        let msg = InstantiateMsg {
            admin: ADMIN.to_string(),
            token_info,
            per_address_limit: Some(5),
            start_time: Some(app.block_info().time.plus_seconds(1)),
            collection_info,
            max_token_limit: Some(0),
            unit_price: Some(coin(100, NATIVE_DENOM)),
            native_denom: NATIVE_DENOM.to_string(),
        };
        let err = app
            .instantiate_contract(
                token_code_id,
                Addr::unchecked(ADMIN),
                &msg,
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
        let token_code_id = app.store_code(token_contract());

        let collection_info = CollectionInfo {
            collection_type: Collections::Normal,
            name: "Test Collection".to_string(),
            description: "Test Description".to_string(),
            image: "https://some-image.com".to_string(),
            external_link: None,
        };
        let token_info = TokenInfo {
            symbol: "TTT".to_string(),
            minter: ADMIN.to_string(),
        };

        let msg = InstantiateMsg {
            admin: ADMIN.to_string(),
            token_info,
            per_address_limit: Some(0),
            start_time: Some(app.block_info().time.plus_seconds(1)),
            collection_info,
            max_token_limit: Some(100),
            unit_price: Some(coin(100, NATIVE_DENOM)),
            native_denom: NATIVE_DENOM.to_string(),
        };
        let err = app
            .instantiate_contract(
                token_code_id,
                Addr::unchecked(ADMIN),
                &msg,
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
    fn test_invalid_description() {
        let mut app = mock_app();
        let token_code_id = app.store_code(token_contract());

        let collection_info = CollectionInfo {
            collection_type: Collections::Normal,
            name: "Test Collection".to_string(),
            description: "Test DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest DescriptionTest Description".to_string(),
            image: "https://some-image.com".to_string(),
            external_link: None,
        };
        let token_info = TokenInfo {
            symbol: "TTT".to_string(),
            minter: ADMIN.to_string(),
        };

        let msg = InstantiateMsg {
            admin: ADMIN.to_string(),
            token_info,
            per_address_limit: Some(5),
            start_time: Some(app.block_info().time.plus_seconds(1)),
            collection_info,
            max_token_limit: Some(100),
            unit_price: Some(coin(100, NATIVE_DENOM)),
            native_denom: NATIVE_DENOM.to_string(),
        };
        let err = app
            .instantiate_contract(
                token_code_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            ContractError::DescriptionTooLong {}.to_string()
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
            let token_contract_addr =
                proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

            let msg = ExecuteMsg::UpdateOperators {
                addrs: vec![RANDOM.to_string(), RANDOM_2.to_string()],
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    token_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let msg = QueryMsg::ContractOperators {};
            let res: ResponseWrapper<Vec<String>> = app
                .wrap()
                .query_wasm_smart(token_contract_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data, vec![RANDOM.to_string(), RANDOM_2.to_string()]);
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let token_contract_addr =
                proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

            let msg = ExecuteMsg::UpdateOperators {
                addrs: vec![RANDOM.to_string(), RANDOM_2.to_string()],
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(USER),
                    token_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }
    }

    mod update_locks {
        use super::*;

        mod operation_lock {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

                let msg = ExecuteMsg::UpdateOperationLock { lock: true };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let res =
                    query_token_operation_lock(&app.wrap(), &token_contract_addr.clone()).unwrap();
                assert_eq!(res, true);
            }

            #[test]
            fn test_invalid_admin() {
                let mut app = mock_app();
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

                let msg = ExecuteMsg::UpdateOperationLock { lock: true };
                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        token_contract_addr.clone(),
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

        mod normal_locks {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

                let locks = Locks {
                    mint_lock: false,
                    burn_lock: true,
                    transfer_lock: true,
                    send_lock: false,
                };
                let msg = ExecuteMsg::UpdateLocks {
                    locks: locks.clone(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = QueryMsg::Locks {};
                let res: ResponseWrapper<Locks> = app
                    .wrap()
                    .query_wasm_smart(token_contract_addr.clone(), &msg)
                    .unwrap();
                assert_eq!(res.data, locks);
            }

            #[test]
            fn test_invalid_admin() {
                let mut app = mock_app();
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

                let locks = Locks {
                    mint_lock: false,
                    burn_lock: true,
                    transfer_lock: true,
                    send_lock: false,
                };
                let msg = ExecuteMsg::UpdateLocks {
                    locks: locks.clone(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
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
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

                let msg = ExecuteMsg::Mint {
                    owner: USER.to_string(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let locks = Locks {
                    mint_lock: false,
                    burn_lock: true,
                    transfer_lock: true,
                    send_lock: false,
                };
                let msg = ExecuteMsg::UpdateTokenLock {
                    token_id: "1".to_string(),
                    locks: locks.clone(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = QueryMsg::TokenLocks {
                    token_id: "1".to_string(),
                };
                let res: ResponseWrapper<Locks> = app
                    .wrap()
                    .query_wasm_smart(token_contract_addr.clone(), &msg)
                    .unwrap();
                assert_eq!(res.data, locks);
            }

            #[test]
            fn test_invalid_admin() {
                let mut app = mock_app();
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

                let locks = Locks {
                    mint_lock: false,
                    burn_lock: true,
                    transfer_lock: true,
                    send_lock: false,
                };
                let msg = ExecuteMsg::UpdateTokenLock {
                    token_id: "1".to_string(),
                    locks: locks.clone(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::Unauthorized {}.to_string()
                );
            }

            #[test]
            fn test_invalid_token_id() {
                let mut app = mock_app();
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

                let locks = Locks {
                    mint_lock: false,
                    burn_lock: true,
                    transfer_lock: true,
                    send_lock: false,
                };
                let msg = ExecuteMsg::UpdateTokenLock {
                    token_id: "1".to_string(),
                    locks: locks.clone(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
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
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

                let msg = ExecuteMsg::UpdatePerAddressLimit {
                    per_address_limit: Some(5),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = QueryMsg::Config {};
                let res: ResponseWrapper<ConfigResponse> = app
                    .wrap()
                    .query_wasm_smart(token_contract_addr.clone(), &msg)
                    .unwrap();
                assert_eq!(res.data.per_address_limit, Some(5));
            }

            #[test]
            fn test_invalid_admin() {
                let mut app = mock_app();
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

                let msg = ExecuteMsg::UpdatePerAddressLimit {
                    per_address_limit: Some(5),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::Unauthorized {}.to_string()
                );
            }

            #[test]
            fn test_invalid_per_address_limit() {
                let mut app = mock_app();
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

                let msg = ExecuteMsg::UpdatePerAddressLimit {
                    per_address_limit: Some(0),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
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
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

                let msg = ExecuteMsg::UpdateStartTime {
                    start_time: Some(app.block_info().time.plus_seconds(5)),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = QueryMsg::Config {};
                let res: ResponseWrapper<ConfigResponse> = app
                    .wrap()
                    .query_wasm_smart(token_contract_addr.clone(), &msg)
                    .unwrap();
                assert_eq!(
                    res.data.start_time,
                    Some(app.block_info().time.plus_seconds(5))
                );
            }

            #[test]
            fn test_invalid_admin() {
                let mut app = mock_app();
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

                let msg = ExecuteMsg::UpdateStartTime {
                    start_time: Some(app.block_info().time.plus_seconds(5)),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
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
                let token_contract_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    Some(start_time),
                    None,
                    None,
                );

                let genesis_time = app.block_info().time;

                app.update_block(|block| block.time = block.time.plus_seconds(10));

                let msg = ExecuteMsg::UpdateStartTime {
                    start_time: Some(app.block_info().time.plus_seconds(5)),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::AlreadyStarted {}.to_string()
                );

                app.update_block(|block| block.time = block.time.minus_seconds(6));

                let msg = ExecuteMsg::UpdateStartTime {
                    start_time: Some(genesis_time.plus_seconds(2)),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
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
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

                let msg = ExecuteMsg::Mint {
                    owner: USER.to_string(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = ExecuteMsg::TransferNft {
                    recipient: RANDOM.to_string(),
                    token_id: "1".to_string(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let res = query_token_owner(&app.wrap(), &token_contract_addr, &1).unwrap();
                assert_eq!(res, Addr::unchecked(RANDOM));
            }

            #[test]
            fn test_invalid_locks() {
                let mut app = mock_app();
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

                let msg = ExecuteMsg::Mint {
                    owner: USER.to_string(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let locks = Locks {
                    transfer_lock: true,
                    mint_lock: false,
                    burn_lock: false,
                    send_lock: false,
                };

                let msg = ExecuteMsg::UpdateTokenLock {
                    token_id: "1".to_string(),
                    locks: locks.clone(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = ExecuteMsg::TransferNft {
                    recipient: RANDOM.to_string(),
                    token_id: "1".to_string(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::TransferLocked {}.to_string()
                );

                let msg = ExecuteMsg::UpdateLocks {
                    locks: locks.clone(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = ExecuteMsg::TransferNft {
                    recipient: RANDOM.to_string(),
                    token_id: "1".to_string(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::TransferLocked {}.to_string()
                );

                let msg = ExecuteMsg::UpdateOperationLock { lock: true };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = ExecuteMsg::TransferNft {
                    recipient: RANDOM.to_string(),
                    token_id: "1".to_string(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
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
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

                let msg = ExecuteMsg::Mint {
                    owner: USER.to_string(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = ExecuteMsg::ApproveAll {
                    operator: ADMIN.to_string(),
                    expires: None,
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = ExecuteMsg::AdminTransferNft {
                    recipient: RANDOM.to_string(),
                    token_id: "1".to_string(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let res = query_token_owner(&app.wrap(), &token_contract_addr, &1).unwrap();
                assert_eq!(res, Addr::unchecked(RANDOM));
            }

            #[test]
            fn test_invalid_admin() {
                let mut app = mock_app();
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

                let msg = ExecuteMsg::Mint {
                    owner: USER.to_string(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
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
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

                let msg = ExecuteMsg::Mint {
                    owner: USER.to_string(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = ExecuteMsg::Burn {
                    token_id: "1".to_string(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let res = query_token_owner(&app.wrap(), &token_contract_addr, &1);
                assert!(res.is_err());
            }

            #[test]
            fn test_invalid_locks() {
                let mut app = mock_app();
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

                let msg = ExecuteMsg::Mint {
                    owner: USER.to_string(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let locks = Locks {
                    transfer_lock: false,
                    mint_lock: false,
                    burn_lock: true,
                    send_lock: false,
                };

                let msg = ExecuteMsg::UpdateTokenLock {
                    token_id: "1".to_string(),
                    locks: locks.clone(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = ExecuteMsg::Burn {
                    token_id: "1".to_string(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::BurnLocked {}.to_string()
                );

                let msg = ExecuteMsg::UpdateLocks {
                    locks: locks.clone(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = ExecuteMsg::Burn {
                    token_id: "1".to_string(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::BurnLocked {}.to_string()
                );

                let msg = ExecuteMsg::UpdateOperationLock { lock: true };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = ExecuteMsg::Burn {
                    token_id: "1".to_string(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::BurnLocked {}.to_string()
                );
            }
        }

        mod mint_operation {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

                let msg = ExecuteMsg::Mint {
                    owner: USER.to_string(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let res = query_token_owner(&app.wrap(), &token_contract_addr, &1).unwrap();
                assert_eq!(res, Addr::unchecked(USER));
            }

            #[test]
            fn test_invalid_locks() {
                let mut app = mock_app();
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, None, None);

                let locks = Locks {
                    mint_lock: true,
                    transfer_lock: false,
                    burn_lock: false,
                    send_lock: false,
                };

                let msg = ExecuteMsg::UpdateLocks {
                    locks: locks.clone(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = ExecuteMsg::Mint {
                    owner: USER.to_string(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::MintLocked {}.to_string()
                );

                let msg = ExecuteMsg::UpdateOperationLock { lock: true };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = ExecuteMsg::Mint {
                    owner: USER.to_string(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::MintLocked {}.to_string()
                );
            }

            #[test]
            fn test_max_token_limit() {
                let mut app = mock_app();
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), None, None, Some(2), None);

                let msg = ExecuteMsg::Mint {
                    owner: USER.to_string(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = ExecuteMsg::Mint {
                    owner: RANDOM.to_string(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = ExecuteMsg::Mint {
                    owner: RANDOM_2.to_string(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::TokenLimitReached {}.to_string()
                );
            }

            #[test]
            fn test_per_address_limit() {
                let mut app = mock_app();
                let token_contract_addr =
                    proper_instantiate(&mut app, ADMIN.to_string(), Some(2), None, None, None);

                let msg = ExecuteMsg::Mint {
                    owner: USER.to_string(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = ExecuteMsg::Mint {
                    owner: USER.to_string(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = ExecuteMsg::Mint {
                    owner: USER.to_string(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
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
                let token_contract_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    Some(start_time),
                    None,
                    None,
                );

                let msg = ExecuteMsg::Mint {
                    owner: USER.to_string(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::MintingNotStarted {}.to_string()
                );
            }

            #[test]
            fn test_invalid_funds() {
                let mut app = mock_app();
                let token_contract_addr = proper_instantiate(
                    &mut app,
                    RANDOM.to_string(),
                    None,
                    None,
                    None,
                    Some(coin(100, NATIVE_DENOM)),
                );

                let msg = ExecuteMsg::Mint {
                    owner: USER.to_string(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(RANDOM),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![coin(100, TEST_DENOM)],
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

                let token_contract_addr = proper_instantiate(
                    &mut app,
                    ADMIN.to_string(),
                    None,
                    None,
                    None,
                    Some(coin(100, NATIVE_DENOM)),
                );

                let msg = ExecuteMsg::Mint {
                    owner: USER.to_string(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    FundsError::MissingFunds {}.to_string()
                );

                let msg = ExecuteMsg::Mint {
                    owner: USER.to_string(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        token_contract_addr.clone(),
                        &msg,
                        &vec![coin(50, NATIVE_DENOM)],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    FundsError::InvalidFunds {
                        got: "50".to_string(),
                        expected: "100".to_string()
                    }
                    .to_string()
                );
            }
        }
    }
}
