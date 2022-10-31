use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    ContractError,
};
use cosmwasm_std::{to_binary, Addr, Coin, Empty, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_types::shared::RegisterMsg;

pub fn marketplace_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

const USER: &str = "juno..user";
const ADMIN: &str = "juno..admin";
const NATIVE_DENOM: &str = "native_denom";

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
    })
}

fn proper_instantiate(app: &mut App) -> Addr {
    let marketplace_code_id = app.store_code(marketplace_module());

    let msg = RegisterMsg {
        admin: ADMIN.to_string(),
        data: Some(
            to_binary(&InstantiateMsg {
                native_denom: NATIVE_DENOM.to_string(),
            })
            .unwrap(),
        ),
    };
    app.instantiate_contract(
        marketplace_code_id,
        Addr::unchecked(ADMIN),
        &msg,
        &[],
        "test",
        None,
    )
    .unwrap()
}

mod instantiate {
    use super::*;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let marketplace_code_id = app.store_code(marketplace_module());

        let msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(
                to_binary(&InstantiateMsg {
                    native_denom: NATIVE_DENOM.to_string(),
                })
                .unwrap(),
            ),
        };
        let _ = app
            .instantiate_contract(
                marketplace_code_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();
    }

    #[test]
    fn test_invalid_msg() {
        let mut app = mock_app();
        let marketplace_code_id = app.store_code(marketplace_module());

        let msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: None,
        };
        let err = app
            .instantiate_contract(
                marketplace_code_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            ContractError::InvalidInstantiateMsg {}.to_string()
        )
    }
}

mod actions {
    use super::*;

    mod update_operators {
        use komple_types::query::ResponseWrapper;

        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let marketplace_module_addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateOperators {
                addrs: vec![
                    "juno..first".to_string(),
                    "juno..second".to_string(),
                    "juno..first".to_string(),
                ],
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    marketplace_module_addr.clone(),
                    &msg,
                    &[],
                )
                .unwrap();

            let msg = QueryMsg::Operators {};
            let res: ResponseWrapper<Vec<String>> = app
                .wrap()
                .query_wasm_smart(marketplace_module_addr.clone(), &msg)
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
                    marketplace_module_addr.clone(),
                    &msg,
                    &[],
                )
                .unwrap();

            let msg = QueryMsg::Operators {};
            let res: ResponseWrapper<Vec<String>> = app
                .wrap()
                .query_wasm_smart(marketplace_module_addr, &msg)
                .unwrap();
            assert_eq!(res.data.len(), 1);
            assert_eq!(res.data[0], "juno..third");
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let marketplace_module_addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateOperators {
                addrs: vec!["juno..first".to_string(), "juno..second".to_string()],
            };
            let err = app
                .execute_contract(Addr::unchecked(USER), marketplace_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }

        #[test]
        fn test_invalid_operator() {
            let mut app = mock_app();
            let marketplace_module_addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateOperators {
                addrs: vec!["juno..first".to_string(), "juno..second".to_string()],
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    marketplace_module_addr.clone(),
                    &msg,
                    &[],
                )
                .unwrap();

            let err = app
                .execute_contract(
                    Addr::unchecked("juno..third"),
                    marketplace_module_addr,
                    &msg,
                    &[],
                )
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
