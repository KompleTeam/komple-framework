use crate::msg::{ConfigResponse, ExecuteMsg, QueryMsg};
use crate::state::WhitelistConfig;
use crate::ContractError;
use cosmwasm_std::{to_binary, Addr, Coin, Empty, Timestamp, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_types::hub::RegisterMsg;
use komple_types::query::ResponseWrapper;

use crate::msg::InstantiateMsg;

pub fn whitelist_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

const USER: &str = "juno..user";
const ADMIN: &str = "juno..admin";
const RANDOM: &str = "juno..random";
const RANDOM_2: &str = "juno..random2";
const RANDOM_3: &str = "juno..random3";
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
                    amount: Uint128::new(1),
                }],
            )
            .unwrap();
    })
}

fn proper_instantiate(
    app: &mut App,
    members: Vec<String>,
    start_time: Timestamp,
    end_time: Timestamp,
    per_address_limit: u8,
    member_limit: u16,
) -> Addr {
    let whitelist_code_id = app.store_code(whitelist_module());

    let msg = InstantiateMsg {
        members,
        config: WhitelistConfig {
            start_time,
            end_time,
            per_address_limit,
            member_limit,
        },
    };
    let register_msg = RegisterMsg {
        admin: ADMIN.to_string(),
        data: Some(to_binary(&msg).unwrap()),
    };

    app.instantiate_contract(
        whitelist_code_id,
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
        let whitelist_code_id = app.store_code(whitelist_module());

        let msg = InstantiateMsg {
            members: vec![RANDOM.to_string()],
            config: WhitelistConfig {
                start_time: app.block_info().time.plus_seconds(1),
                end_time: app.block_info().time.plus_seconds(10),
                per_address_limit: 5,
                member_limit: 10,
            },
        };
        let register_msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(to_binary(&msg).unwrap()),
        };
        let _ = app
            .instantiate_contract(
                whitelist_code_id,
                Addr::unchecked(ADMIN),
                &register_msg,
                &[],
                "test",
                None,
            )
            .unwrap();
    }

    #[test]
    fn test_invalid_member_limit() {
        let mut app = mock_app();
        let whitelist_code_id = app.store_code(whitelist_module());

        let msg = InstantiateMsg {
            members: vec![RANDOM.to_string()],
            config: WhitelistConfig {
                start_time: app.block_info().time,
                end_time: app.block_info().time.plus_seconds(10),
                per_address_limit: 5,
                member_limit: 0,
            },
        };
        let register_msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(to_binary(&msg).unwrap()),
        };
        let err = app
            .instantiate_contract(
                whitelist_code_id,
                Addr::unchecked(ADMIN),
                &register_msg,
                &[],
                "test",
                None,
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            ContractError::InvalidMemberLimit {}.to_string()
        );
    }

    #[test]
    fn test_invalid_per_address_limit() {
        let mut app = mock_app();
        let whitelist_code_id = app.store_code(whitelist_module());

        let msg = InstantiateMsg {
            members: vec![RANDOM.to_string()],
            config: WhitelistConfig {
                start_time: app.block_info().time,
                end_time: app.block_info().time.plus_seconds(10),
                per_address_limit: 0,
                member_limit: 10,
            },
        };
        let register_msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(to_binary(&msg).unwrap()),
        };
        let err = app
            .instantiate_contract(
                whitelist_code_id,
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
    fn test_invalid_member_list() {
        let mut app = mock_app();
        let whitelist_code_id = app.store_code(whitelist_module());

        let msg = InstantiateMsg {
            members: vec![],
            config: WhitelistConfig {
                start_time: app.block_info().time,
                end_time: app.block_info().time.plus_seconds(10),
                per_address_limit: 5,
                member_limit: 10,
            },
        };
        let register_msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(to_binary(&msg).unwrap()),
        };
        let err = app
            .instantiate_contract(
                whitelist_code_id,
                Addr::unchecked(ADMIN),
                &register_msg,
                &[],
                "test",
                None,
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            ContractError::EmptyMemberList {}.to_string()
        );
    }

    #[test]
    fn test_invalid_times() {
        let mut app = mock_app();
        let whitelist_code_id = app.store_code(whitelist_module());

        let msg = InstantiateMsg {
            members: vec![RANDOM.to_string()],
            config: WhitelistConfig {
                start_time: app.block_info().time.minus_seconds(10),
                end_time: app.block_info().time.plus_seconds(10),
                per_address_limit: 5,
                member_limit: 10,
            },
        };
        let register_msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(to_binary(&msg).unwrap()),
        };
        let err = app
            .instantiate_contract(
                whitelist_code_id,
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

        let msg = InstantiateMsg {
            members: vec![RANDOM.to_string()],
            config: WhitelistConfig {
                start_time: app.block_info().time,
                end_time: app.block_info().time.plus_seconds(10),
                per_address_limit: 5,
                member_limit: 10,
            },
        };
        let register_msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(to_binary(&msg).unwrap()),
        };
        let err = app
            .instantiate_contract(
                whitelist_code_id,
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

        let msg = InstantiateMsg {
            members: vec![RANDOM.to_string()],
            config: WhitelistConfig {
                start_time: app.block_info().time.plus_seconds(1),
                end_time: app.block_info().time.minus_seconds(10),
                per_address_limit: 5,
                member_limit: 10,
            },
        };
        let register_msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(to_binary(&msg).unwrap()),
        };
        let err = app
            .instantiate_contract(
                whitelist_code_id,
                Addr::unchecked(ADMIN),
                &register_msg,
                &[],
                "test",
                None,
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            ContractError::InvalidEndTime {}.to_string()
        );

        let msg = InstantiateMsg {
            members: vec![RANDOM.to_string()],
            config: WhitelistConfig {
                start_time: app.block_info().time.plus_seconds(10),
                end_time: app.block_info().time.plus_seconds(10),
                per_address_limit: 5,
                member_limit: 10,
            },
        };
        let register_msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(to_binary(&msg).unwrap()),
        };
        let err = app
            .instantiate_contract(
                whitelist_code_id,
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

        let msg = InstantiateMsg {
            members: vec![RANDOM.to_string()],
            config: WhitelistConfig {
                start_time: app.block_info().time.plus_seconds(15),
                end_time: app.block_info().time.plus_seconds(10),
                per_address_limit: 5,
                member_limit: 10,
            },
        };
        let register_msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(to_binary(&msg).unwrap()),
        };
        let err = app
            .instantiate_contract(
                whitelist_code_id,
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
}

mod actions {
    use super::*;

    mod update_times {
        use super::*;

        mod update_start_time {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let mut whitelist_config = WhitelistConfig {
                    start_time,
                    end_time,
                    per_address_limit: 5,
                    member_limit: 10,
                };
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string()],
                    whitelist_config.start_time,
                    whitelist_config.end_time,
                    whitelist_config.per_address_limit,
                    whitelist_config.member_limit,
                );

                whitelist_config.start_time = app.block_info().time.plus_seconds(5);
                let msg = ExecuteMsg::UpdateWhitelistConfig { whitelist_config };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        whitelist_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();

                let msg = QueryMsg::Config {};
                let res: ResponseWrapper<ConfigResponse> = app
                    .wrap()
                    .query_wasm_smart(whitelist_module_addr, &msg)
                    .unwrap();
                assert_eq!(res.data.start_time, app.block_info().time.plus_seconds(5));
            }

            #[test]
            fn test_invalid_admin() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let mut whitelist_config = WhitelistConfig {
                    start_time,
                    end_time,
                    per_address_limit: 5,
                    member_limit: 10,
                };
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string()],
                    whitelist_config.start_time,
                    whitelist_config.end_time,
                    whitelist_config.per_address_limit,
                    whitelist_config.member_limit,
                );

                whitelist_config.start_time = app.block_info().time.plus_seconds(5);
                let msg = ExecuteMsg::UpdateWhitelistConfig { whitelist_config };
                let err = app
                    .execute_contract(Addr::unchecked(USER), whitelist_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::Unauthorized {}.to_string()
                );
            }

            #[test]
            fn test_invalid_times() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let mut whitelist_config = WhitelistConfig {
                    start_time,
                    end_time,
                    per_address_limit: 5,
                    member_limit: 10,
                };
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string()],
                    whitelist_config.start_time,
                    whitelist_config.end_time,
                    whitelist_config.per_address_limit,
                    whitelist_config.member_limit,
                );

                whitelist_config.start_time = app.block_info().time.minus_seconds(10);
                let msg = ExecuteMsg::UpdateWhitelistConfig {
                    whitelist_config: whitelist_config.clone(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        whitelist_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::InvalidStartTime {}.to_string()
                );

                whitelist_config.start_time = app.block_info().time;
                let msg = ExecuteMsg::UpdateWhitelistConfig {
                    whitelist_config: whitelist_config.clone(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        whitelist_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::InvalidStartTime {}.to_string()
                );

                whitelist_config.start_time = app.block_info().time.plus_seconds(11);
                let msg = ExecuteMsg::UpdateWhitelistConfig {
                    whitelist_config: whitelist_config.clone(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        whitelist_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::InvalidStartTime {}.to_string()
                );

                app.update_block(|block| block.time = block.time.plus_seconds(5));

                whitelist_config.start_time = app.block_info().time.plus_seconds(11);
                let msg = ExecuteMsg::UpdateWhitelistConfig { whitelist_config };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), whitelist_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::AlreadyStarted {}.to_string()
                );
            }
        }

        mod update_end_time {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let mut whitelist_config = WhitelistConfig {
                    start_time,
                    end_time,
                    per_address_limit: 5,
                    member_limit: 10,
                };
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string()],
                    whitelist_config.start_time,
                    whitelist_config.end_time,
                    whitelist_config.per_address_limit,
                    whitelist_config.member_limit,
                );

                whitelist_config.end_time = app.block_info().time.plus_seconds(5);
                let msg = ExecuteMsg::UpdateWhitelistConfig { whitelist_config };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        whitelist_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();

                let msg = QueryMsg::Config {};
                let res: ResponseWrapper<ConfigResponse> = app
                    .wrap()
                    .query_wasm_smart(whitelist_module_addr, &msg)
                    .unwrap();
                assert_eq!(res.data.end_time, app.block_info().time.plus_seconds(5));
            }

            #[test]
            fn test_invalid_admin() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(5);
                let end_time = app.block_info().time.plus_seconds(10);
                let mut whitelist_config = WhitelistConfig {
                    start_time,
                    end_time,
                    per_address_limit: 5,
                    member_limit: 10,
                };
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string()],
                    whitelist_config.start_time,
                    whitelist_config.end_time,
                    whitelist_config.per_address_limit,
                    whitelist_config.member_limit,
                );

                whitelist_config.end_time = app.block_info().time.plus_seconds(8);
                let msg = ExecuteMsg::UpdateWhitelistConfig { whitelist_config };
                let err = app
                    .execute_contract(Addr::unchecked(USER), whitelist_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::Unauthorized {}.to_string()
                );
            }

            #[test]
            fn test_invalid_times() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let mut whitelist_config = WhitelistConfig {
                    start_time,
                    end_time,
                    per_address_limit: 5,
                    member_limit: 10,
                };
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string()],
                    whitelist_config.start_time,
                    whitelist_config.end_time,
                    whitelist_config.per_address_limit,
                    whitelist_config.member_limit,
                );

                whitelist_config.end_time = app.block_info().time.minus_seconds(10);
                let msg = ExecuteMsg::UpdateWhitelistConfig {
                    whitelist_config: whitelist_config.clone(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        whitelist_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::InvalidEndTime {}.to_string()
                );

                whitelist_config.end_time = app.block_info().time;
                let msg = ExecuteMsg::UpdateWhitelistConfig {
                    whitelist_config: whitelist_config.clone(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        whitelist_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::InvalidEndTime {}.to_string()
                );

                whitelist_config.end_time = app.block_info().time.plus_seconds(1);
                let msg = ExecuteMsg::UpdateWhitelistConfig {
                    whitelist_config: whitelist_config.clone(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        whitelist_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::InvalidEndTime {}.to_string()
                );

                app.update_block(|block| block.time = block.time.plus_seconds(5));

                whitelist_config.end_time = app.block_info().time.plus_seconds(11);
                let msg = ExecuteMsg::UpdateWhitelistConfig { whitelist_config };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), whitelist_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::AlreadyStarted {}.to_string()
                );
            }
        }
    }

    mod members {
        use super::*;

        mod add_members {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string(), RANDOM_3.to_string(), RANDOM.to_string()],
                    start_time,
                    end_time,
                    5,
                    10,
                );

                let msg = ExecuteMsg::AddMembers {
                    members: vec![RANDOM_2.to_string()],
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        whitelist_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();

                let msg = QueryMsg::Members {
                    start_after: None,
                    limit: None,
                };
                let res: ResponseWrapper<Vec<String>> = app
                    .wrap()
                    .query_wasm_smart(whitelist_module_addr, &msg)
                    .unwrap();
                assert_eq!(
                    res.data,
                    vec![
                        RANDOM.to_string(),
                        RANDOM_2.to_string(),
                        RANDOM_3.to_string()
                    ]
                );
            }

            #[test]
            fn test_invalid_admin() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string()],
                    start_time,
                    end_time,
                    5,
                    10,
                );

                let msg = ExecuteMsg::AddMembers {
                    members: vec![RANDOM_2.to_string()],
                };
                let err = app
                    .execute_contract(Addr::unchecked(USER), whitelist_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::Unauthorized {}.to_string()
                );
            }

            #[test]
            fn test_invalid_time() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string()],
                    start_time,
                    end_time,
                    5,
                    10,
                );

                app.update_block(|block| block.time = block.time.plus_seconds(5));

                let msg = ExecuteMsg::AddMembers {
                    members: vec![RANDOM_2.to_string()],
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), whitelist_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::AlreadyStarted {}.to_string()
                );
            }

            #[test]
            fn test_existing_member() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string()],
                    start_time,
                    end_time,
                    5,
                    10,
                );

                let msg = ExecuteMsg::AddMembers {
                    members: vec![RANDOM.to_string()],
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), whitelist_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::MemberExists {}.to_string()
                );
            }

            #[test]
            fn test_invalid_member_limit() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string(), RANDOM_2.to_string()],
                    start_time,
                    end_time,
                    5,
                    2,
                );

                let msg = ExecuteMsg::AddMembers {
                    members: vec![RANDOM_3.to_string()],
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), whitelist_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::MemberLimitExceeded {}.to_string()
                );
            }
        }

        mod remove_members {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![
                        RANDOM.to_string(),
                        RANDOM_3.to_string(),
                        RANDOM_2.to_string(),
                    ],
                    start_time,
                    end_time,
                    5,
                    10,
                );

                let msg = ExecuteMsg::RemoveMembers {
                    members: vec![RANDOM_2.to_string()],
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        whitelist_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();

                let msg = QueryMsg::Members {
                    start_after: None,
                    limit: None,
                };
                let res: ResponseWrapper<Vec<String>> = app
                    .wrap()
                    .query_wasm_smart(whitelist_module_addr, &msg)
                    .unwrap();
                assert_eq!(res.data, vec![RANDOM.to_string(), RANDOM_3.to_string()]);
            }

            #[test]
            fn test_invalid_admin() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string()],
                    start_time,
                    end_time,
                    5,
                    10,
                );

                let msg = ExecuteMsg::RemoveMembers {
                    members: vec![RANDOM_2.to_string()],
                };
                let err = app
                    .execute_contract(Addr::unchecked(USER), whitelist_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::Unauthorized {}.to_string()
                );
            }

            #[test]
            fn test_invalid_time() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string()],
                    start_time,
                    end_time,
                    5,
                    10,
                );

                app.update_block(|block| block.time = block.time.plus_seconds(5));

                let msg = ExecuteMsg::RemoveMembers {
                    members: vec![RANDOM_2.to_string()],
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), whitelist_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::AlreadyStarted {}.to_string()
                );
            }

            #[test]
            fn test_missing_member() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string()],
                    start_time,
                    end_time,
                    5,
                    10,
                );

                let msg = ExecuteMsg::RemoveMembers {
                    members: vec![RANDOM_2.to_string()],
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), whitelist_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::MemberNotFound {}.to_string()
                );
            }
        }
    }

    mod limits {
        use super::*;

        mod update_per_address_limit {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let mut whitelist_config = WhitelistConfig {
                    start_time,
                    end_time,
                    per_address_limit: 5,
                    member_limit: 10,
                };
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string()],
                    whitelist_config.start_time,
                    whitelist_config.end_time,
                    whitelist_config.per_address_limit,
                    whitelist_config.member_limit,
                );

                whitelist_config.per_address_limit = 10;
                let msg = ExecuteMsg::UpdateWhitelistConfig { whitelist_config };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        whitelist_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();

                let msg = QueryMsg::Config {};
                let res: ResponseWrapper<ConfigResponse> = app
                    .wrap()
                    .query_wasm_smart(whitelist_module_addr, &msg)
                    .unwrap();
                assert_eq!(res.data.per_address_limit, 10);
            }

            #[test]
            fn test_invalid_admin() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let mut whitelist_config = WhitelistConfig {
                    start_time,
                    end_time,
                    per_address_limit: 5,
                    member_limit: 10,
                };
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string()],
                    whitelist_config.start_time,
                    whitelist_config.end_time,
                    whitelist_config.per_address_limit,
                    whitelist_config.member_limit,
                );

                whitelist_config.per_address_limit = 10;
                let msg = ExecuteMsg::UpdateWhitelistConfig { whitelist_config };
                let err = app
                    .execute_contract(Addr::unchecked(USER), whitelist_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::Unauthorized {}.to_string()
                );
            }

            #[test]
            fn test_invalid_time() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let mut whitelist_config = WhitelistConfig {
                    start_time,
                    end_time,
                    per_address_limit: 5,
                    member_limit: 10,
                };
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string()],
                    whitelist_config.start_time,
                    whitelist_config.end_time,
                    whitelist_config.per_address_limit,
                    whitelist_config.member_limit,
                );

                app.update_block(|block| block.time = block.time.plus_seconds(5));

                whitelist_config.per_address_limit = 10;
                let msg = ExecuteMsg::UpdateWhitelistConfig { whitelist_config };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), whitelist_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::AlreadyStarted {}.to_string()
                );
            }

            #[test]
            fn test_invalid_limit() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let mut whitelist_config = WhitelistConfig {
                    start_time,
                    end_time,
                    per_address_limit: 5,
                    member_limit: 10,
                };
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string()],
                    whitelist_config.start_time,
                    whitelist_config.end_time,
                    whitelist_config.per_address_limit,
                    whitelist_config.member_limit,
                );

                whitelist_config.per_address_limit = 0;
                let msg = ExecuteMsg::UpdateWhitelistConfig { whitelist_config };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), whitelist_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::InvalidPerAddressLimit {}.to_string()
                );
            }
        }

        mod update_member_limit {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let mut whitelist_config = WhitelistConfig {
                    start_time,
                    end_time,
                    per_address_limit: 5,
                    member_limit: 10,
                };
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string()],
                    whitelist_config.start_time,
                    whitelist_config.end_time,
                    whitelist_config.per_address_limit,
                    whitelist_config.member_limit,
                );

                whitelist_config.member_limit = 20;
                let msg = ExecuteMsg::UpdateWhitelistConfig { whitelist_config };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        whitelist_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();

                let msg = QueryMsg::Config {};
                let res: ResponseWrapper<ConfigResponse> = app
                    .wrap()
                    .query_wasm_smart(whitelist_module_addr, &msg)
                    .unwrap();
                assert_eq!(res.data.member_limit, 20);
            }

            #[test]
            fn test_invalid_admin() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let mut whitelist_config = WhitelistConfig {
                    start_time,
                    end_time,
                    per_address_limit: 5,
                    member_limit: 10,
                };
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string()],
                    whitelist_config.start_time,
                    whitelist_config.end_time,
                    whitelist_config.per_address_limit,
                    whitelist_config.member_limit,
                );

                whitelist_config.member_limit = 20;
                let msg = ExecuteMsg::UpdateWhitelistConfig { whitelist_config };
                let err = app
                    .execute_contract(Addr::unchecked(USER), whitelist_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::Unauthorized {}.to_string()
                );
            }

            #[test]
            fn test_invalid_time() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let mut whitelist_config = WhitelistConfig {
                    start_time,
                    end_time,
                    per_address_limit: 5,
                    member_limit: 10,
                };
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string()],
                    whitelist_config.start_time,
                    whitelist_config.end_time,
                    whitelist_config.per_address_limit,
                    whitelist_config.member_limit,
                );

                app.update_block(|block| block.time = block.time.plus_seconds(5));

                whitelist_config.member_limit = 20;
                let msg = ExecuteMsg::UpdateWhitelistConfig { whitelist_config };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), whitelist_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::AlreadyStarted {}.to_string()
                );
            }

            #[test]
            fn test_invalid_limit() {
                let mut app = mock_app();
                let start_time = app.block_info().time.plus_seconds(1);
                let end_time = app.block_info().time.plus_seconds(10);
                let mut whitelist_config = WhitelistConfig {
                    start_time,
                    end_time,
                    per_address_limit: 5,
                    member_limit: 10,
                };
                let whitelist_module_addr = proper_instantiate(
                    &mut app,
                    vec![RANDOM.to_string()],
                    whitelist_config.start_time,
                    whitelist_config.end_time,
                    whitelist_config.per_address_limit,
                    whitelist_config.member_limit,
                );

                whitelist_config.member_limit = 0;
                let msg = ExecuteMsg::UpdateWhitelistConfig { whitelist_config };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), whitelist_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::InvalidMemberLimit {}.to_string()
                );
            }
        }
    }
}
