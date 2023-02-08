use cosmwasm_std::{Addr, Coin, Empty, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

// use crate::ContractError;
// use crate::{
//     msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
// };

use komple_framework_types::shared::RegisterMsg;

/* TODO: Replace module name here */
pub fn custom_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

// const USER: &str = "juno...user";
const ADMIN: &str = "juno...admin";
const NATIVE_DENOM: &str = "denom";

/* TODO: Define balances here */
fn mock_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(ADMIN),
                vec![Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(100_000_000),
                }],
            )
            .unwrap();
        // router
        //     .bank
        //     .init_balance(
        //         storage,
        //         &Addr::unchecked(USER),
        //         vec![Coin {
        //             denom: NATIVE_DENOM.to_string(),
        //             amount: Uint128::new(100_000_000),
        //         }],
        //     )
        //     .unwrap();
    })
}

fn proper_instantiate(app: &mut App) -> Addr {
    let custom_code_id = app.store_code(custom_module());

    let msg = RegisterMsg {
        admin: ADMIN.to_string(),
        data: None,
    };

    app.instantiate_contract(
        custom_code_id,
        Addr::unchecked(ADMIN),
        &msg,
        &[],
        "test",
        None,
    )
    .unwrap()
}

/* TODO: Add extra helpers and methods here */
/* ... */

mod initialization {
    use super::*;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let custom_code_id = app.store_code(custom_module());

        let msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: None,
        };
        let _ = app
            .instantiate_contract(
                custom_code_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();
    }
}

mod actions {
    use super::*;

    mod execute_message_1 {
        use super::*;

        /* TODO: Add your tests here */
        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let custom_addr = proper_instantiate(&mut app);
        }
    }

    mod execute_message_2 {
        use super::*;

        /* TODO: Add your tests here */
        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let custom_addr = proper_instantiate(&mut app);
        }
    }

    /* Add more execute messages here */
    /* ... */
}

mod queries {
    use super::*;

    mod query_message_1 {
        use super::*;

        /* TODO: Add your tests here */
        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let custom_addr = proper_instantiate(&mut app);
        }
    }

    mod query_message_2 {
        use super::*;

        /* TODO: Add your tests here */
        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let custom_addr = proper_instantiate(&mut app);
        }
    }

    /* TODO: Add more query messages here */
    /* ... */
}
