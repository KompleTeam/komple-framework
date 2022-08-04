#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use crate::msg::InstantiateMsg;

    pub fn passcard_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    const USER: &str = "juno1shfqtuup76mngspx29gcquykjvvlx9na4kymlm";
    const ADMIN: &str = "juno1qamfln8u5w8d3vlhp5t9mhmylfkgad4jz6t7cv";
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

    fn proper_instantiate() -> (App, Addr) {
        let mut app = mock_app();
        let passcard_code_id = app.store_code(passcard_contract());

        let msg = InstantiateMsg {
            controller_address: "juno19pjtx7dah2fquf7udyxjv94h0eraha78nyj9w4".to_string(),
        };
        let passcard_contract_addr = app
            .instantiate_contract(
                passcard_code_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        (app, passcard_contract_addr)
    }

    mod passcards {
        use super::*;

        use crate::{
            msg::{ExecuteMsg, QueryMsg},
            state::{PasscardInfo},
        };

        #[test]
        fn test_add_passcard_happy_path() {
            let (mut app, passcard_addr) = proper_instantiate();

            let passcard_info = PasscardInfo {
                name: "Test Passcard".to_string(),
                description: "Test Description".to_string(),
                image: "ipfs://xyz".to_string(),
                external_link: None,
                total_num: 2,
            };
            let msg = ExecuteMsg::AddPasscard {
                collection_id: 1,
                base_price: Uint128::new(100),
                passcard_info: passcard_info.clone(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), passcard_addr.clone(), &msg, &vec![])
                .unwrap();

            let msg = QueryMsg::GetPasscard {
                collection_id: 1,
                passcard_id: 2,
            };
            let response: Result<Empty, cosmwasm_std::StdError> =
                app.wrap().query_wasm_smart(passcard_addr.clone(), &msg);
            assert!(response.is_ok());

            let msg = QueryMsg::GetPasscard {
                collection_id: 1,
                passcard_id: 3,
            };
            let response: Result<Empty, cosmwasm_std::StdError> =
                app.wrap().query_wasm_smart(passcard_addr.clone(), &msg);
            assert!(response.is_err());
            // assert_eq!(response.err().unwrap().to_string(), "Passcard not found");

            let msg = ExecuteMsg::AddPasscard {
                collection_id: 1,
                base_price: Uint128::new(100),
                passcard_info,
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), passcard_addr.clone(), &msg, &vec![])
                .unwrap();

            let msg = QueryMsg::GetPasscard {
                collection_id: 1,
                passcard_id: 3,
            };
            let response: Result<Empty, cosmwasm_std::StdError> =
                app.wrap().query_wasm_smart(passcard_addr.clone(), &msg);
            assert!(response.is_ok());

            let msg = QueryMsg::GetPasscard {
                collection_id: 1,
                passcard_id: 5,
            };
            let response: Result<Empty, cosmwasm_std::StdError> =
                app.wrap().query_wasm_smart(passcard_addr, &msg);
            assert!(response.is_err());
        }
    }
}
