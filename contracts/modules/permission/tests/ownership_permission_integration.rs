use cosmwasm_std::to_binary;
use cosmwasm_std::{Addr, Coin, Empty, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_framework_permission_module::msg::{ExecuteMsg, QueryMsg};
use komple_types::modules::permission::Permissions;
use komple_types::shared::RegisterMsg;

pub fn permission_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_framework_permission_module::contract::execute,
        komple_framework_permission_module::contract::instantiate,
        komple_framework_permission_module::contract::query,
    )
    .with_reply(komple_framework_permission_module::contract::reply);
    Box::new(contract)
}

pub fn ownership_permission_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_ownership_permission_module::contract::execute,
        komple_ownership_permission_module::contract::instantiate,
        komple_ownership_permission_module::contract::query,
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
    let permission_code_id = app.store_code(permission_module());

    let msg = RegisterMsg {
        admin: ADMIN.to_string(),
        data: None,
    };

    app.instantiate_contract(
        permission_code_id,
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

    mod register_permission {
        use komple_types::shared::query::ResponseWrapper;

        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let permission_module_addr = proper_instantiate(&mut app);

            let ownership_permission_code_id = app.store_code(ownership_permission_module());
            let msg = ExecuteMsg::RegisterPermission {
                permission: Permissions::Ownership.to_string(),
                msg: Some(
                    to_binary(&RegisterMsg {
                        admin: ADMIN.to_string(),
                        data: None,
                    })
                    .unwrap(),
                ),
                code_id: ownership_permission_code_id,
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    permission_module_addr.clone(),
                    &msg,
                    &[],
                )
                .unwrap();

            let msg = QueryMsg::PermissionAddress {
                permission: Permissions::Ownership.to_string(),
            };
            let res: ResponseWrapper<String> = app
                .wrap()
                .query_wasm_smart(permission_module_addr, &msg)
                .unwrap();
            assert_eq!(res.data, "contract1");

            let res = app.wrap().query_wasm_contract_info("contract1").unwrap();
            assert_eq!(res.admin, Some(ADMIN.to_string()));
        }
    }
}
