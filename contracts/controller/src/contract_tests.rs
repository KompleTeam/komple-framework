use crate::msg::InstantiateMsg;
use cosmwasm_std::{Addr, Coin, Empty, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

pub fn controller_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(crate::contract::reply);
    Box::new(contract)
}

pub fn mint_module_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        mint_module::contract::execute,
        mint_module::contract::instantiate,
        mint_module::contract::query,
    )
    .with_reply(mint_module::contract::reply);
    Box::new(contract)
}

pub fn permission_module_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        permission_module::contract::execute,
        permission_module::contract::instantiate,
        permission_module::contract::query,
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

fn proper_instantiate(app: &mut App) -> Addr {
    let controller_code_id = app.store_code(controller_contract());

    let msg = InstantiateMsg {
        name: "Test Controller".to_string(),
        description: "Test Controller".to_string(),
        image: "https://image.com".to_string(),
        external_link: None,
    };
    let controller_contract_addr = app
        .instantiate_contract(
            controller_code_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "test",
            None,
        )
        .unwrap();

    controller_contract_addr
}

mod modules {
    use super::*;

    mod mint_module_tests {
        use super::*;

        use crate::{
            msg::{ExecuteMsg, QueryMsg},
            ContractError,
        };

        use rift_types::{module::Modules, query::AddressResponse};

        #[test]
        fn test_init_happy_path() {
            let mut app = mock_app();
            let controller_contract_addr = proper_instantiate(&mut app);
            let mint_module_code_id = app.store_code(mint_module_contract());

            let msg = ExecuteMsg::InitMintModule {
                code_id: mint_module_code_id,
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    controller_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let msg = QueryMsg::ModuleAddress(Modules::MintModule);
            let res: AddressResponse = app
                .wrap()
                .query_wasm_smart(controller_contract_addr, &msg)
                .unwrap();
            assert_eq!(res.address, "contract1")
        }

        #[test]
        fn test_init_unhappy_path() {
            let mut app = mock_app();
            let controller_contract_addr = proper_instantiate(&mut app);
            let mint_module_code_id = app.store_code(mint_module_contract());

            let msg = ExecuteMsg::InitMintModule {
                code_id: mint_module_code_id,
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(USER),
                    controller_contract_addr.clone(),
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

    mod permission_module_tests {
        use super::*;

        use crate::{
            msg::{ExecuteMsg, QueryMsg},
            ContractError,
        };

        use rift_types::{module::Modules, query::AddressResponse};

        #[test]
        fn test_init_module() {
            let mut app = mock_app();
            let controller_contract_addr = proper_instantiate(&mut app);
            let permission_module_code_id = app.store_code(permission_module_contract());

            let msg = ExecuteMsg::InitPermissionModule {
                code_id: permission_module_code_id,
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    controller_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let msg = QueryMsg::ModuleAddress(Modules::PermissionModule);
            let res: AddressResponse = app
                .wrap()
                .query_wasm_smart(controller_contract_addr, &msg)
                .unwrap();
            assert_eq!(res.address, "contract1")
        }

        #[test]
        fn test_init_unhappy_path() {
            let mut app = mock_app();
            let controller_contract_addr = proper_instantiate(&mut app);
            let permission_module_code_id = app.store_code(permission_module_contract());

            let msg = ExecuteMsg::InitPermissionModule {
                code_id: permission_module_code_id,
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(USER),
                    controller_contract_addr.clone(),
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
}
