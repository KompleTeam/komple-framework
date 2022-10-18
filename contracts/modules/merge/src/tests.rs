use crate::msg::InstantiateMsg;
use crate::{
    msg::{ExecuteMsg, QueryMsg},
    state::Config,
    ContractError,
};
use cosmwasm_std::{Addr, Coin, Empty, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_types::query::ResponseWrapper;

pub fn merge_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

const USER: &str = "juno1shfqtuup76mngspx29gcquykjvvlx9na4kymlm";
const ADMIN: &str = "juno1qamfln8u5w8d3vlhp5t9mhmylfkgad4jz6t7cv";
const RANDOM: &str = "juno1et88c8yd6xr8azkmp02lxtctkqq36lt63tdt7e";
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

fn proper_instantiate(app: &mut App) -> Addr {
    let merge_code_id = app.store_code(merge_module());

    let msg = InstantiateMsg {
        admin: ADMIN.to_string(),
    };

    app.instantiate_contract(
        merge_code_id,
        Addr::unchecked(ADMIN),
        &msg,
        &[],
        "test",
        None,
    )
    .unwrap()
}

mod merge_lock {
    use super::*;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let merge_module_addr = proper_instantiate(&mut app);

        let msg = ExecuteMsg::UpdateMergeLock { lock: true };
        let _ = app
            .execute_contract(Addr::unchecked(ADMIN), merge_module_addr.clone(), &msg, &[])
            .unwrap();

        let msg = QueryMsg::Config {};
        let res: ResponseWrapper<Config> = app
            .wrap()
            .query_wasm_smart(merge_module_addr.clone(), &msg)
            .unwrap();
        assert_eq!(res.data.merge_lock, true);

        let msg = ExecuteMsg::UpdateOperators {
            addrs: vec![USER.to_string()],
        };
        let _ = app
            .execute_contract(Addr::unchecked(ADMIN), merge_module_addr.clone(), &msg, &[])
            .unwrap();

        let msg = ExecuteMsg::UpdateMergeLock { lock: false };
        let _ = app
            .execute_contract(Addr::unchecked(USER), merge_module_addr.clone(), &msg, &[])
            .unwrap();

        let msg = QueryMsg::Config {};
        let res: ResponseWrapper<Config> = app
            .wrap()
            .query_wasm_smart(merge_module_addr, &msg)
            .unwrap();
        assert_eq!(res.data.merge_lock, false);
    }

    #[test]
    fn test_unhappy_path() {
        let mut app = mock_app();
        let merge_module_addr = proper_instantiate(&mut app);

        let msg = ExecuteMsg::UpdateMergeLock { lock: true };
        let err = app
            .execute_contract(Addr::unchecked(USER), merge_module_addr, &msg, &[])
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            ContractError::Unauthorized {}.to_string()
        );
    }
}

mod whitelist_addresses {
    use super::*;

    #[test]
    fn test_update_happy_path() {
        let mut app = mock_app();
        let merge_module_addr = proper_instantiate(&mut app);

        let msg = ExecuteMsg::UpdateOperators {
            addrs: vec![RANDOM.to_string()],
        };
        let _ = app
            .execute_contract(Addr::unchecked(ADMIN), merge_module_addr.clone(), &msg, &[])
            .unwrap();

        let msg = QueryMsg::Operators {};
        let res: ResponseWrapper<Vec<String>> = app
            .wrap()
            .query_wasm_smart(merge_module_addr, &msg)
            .unwrap();
        assert_eq!(res.data, vec![RANDOM.to_string()]);
    }

    #[test]
    fn test_update_unhappy_path() {
        let mut app = mock_app();
        let merge_module_addr = proper_instantiate(&mut app);

        let msg = ExecuteMsg::UpdateOperators {
            addrs: vec![RANDOM.to_string()],
        };
        let err = app
            .execute_contract(Addr::unchecked(USER), merge_module_addr, &msg, &[])
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            ContractError::Unauthorized {}.to_string()
        );
    }
}

mod update_operators {
    use super::*;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let merge_module_addr = proper_instantiate(&mut app);

        let msg = ExecuteMsg::UpdateOperators {
            addrs: vec![
                "juno..first".to_string(),
                "juno..second".to_string(),
                "juno..first".to_string(),
            ],
        };
        let _ = app
            .execute_contract(Addr::unchecked(ADMIN), merge_module_addr.clone(), &msg, &[])
            .unwrap();

        let msg = QueryMsg::Operators {};
        let res: ResponseWrapper<Vec<String>> = app
            .wrap()
            .query_wasm_smart(merge_module_addr.clone(), &msg)
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
                merge_module_addr.clone(),
                &msg,
                &[],
            )
            .unwrap();

        let msg = QueryMsg::Operators {};
        let res: ResponseWrapper<Vec<String>> = app
            .wrap()
            .query_wasm_smart(merge_module_addr, &msg)
            .unwrap();
        assert_eq!(res.data.len(), 1);
        assert_eq!(res.data[0], "juno..third");
    }

    #[test]
    fn test_invalid_admin() {
        let mut app = mock_app();
        let merge_module_addr = proper_instantiate(&mut app);

        let msg = ExecuteMsg::UpdateOperators {
            addrs: vec!["juno..first".to_string(), "juno..second".to_string()],
        };
        let err = app
            .execute_contract(Addr::unchecked(USER), merge_module_addr, &msg, &[])
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            ContractError::Unauthorized {}.to_string()
        );
    }

    #[test]
    fn test_invalid_operator() {
        let mut app = mock_app();
        let merge_module_addr = proper_instantiate(&mut app);

        let msg = ExecuteMsg::UpdateOperators {
            addrs: vec!["juno..first".to_string(), "juno..second".to_string()],
        };
        let _ = app
            .execute_contract(Addr::unchecked(ADMIN), merge_module_addr.clone(), &msg, &[])
            .unwrap();

        let err = app
            .execute_contract(Addr::unchecked("juno..third"), merge_module_addr, &msg, &[])
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            ContractError::Unauthorized {}.to_string()
        );
    }
}
