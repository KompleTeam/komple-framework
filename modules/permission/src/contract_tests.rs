#[cfg(test)]
mod tests {
    use crate::msg::InstantiateMsg;
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn permission_module() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
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
                        amount: Uint128::new(1),
                    }],
                )
                .unwrap();
        })
    }

    fn proper_instantiate(app: &mut App) -> Addr {
        let permission_code_id = app.store_code(permission_module());

        let msg = InstantiateMsg {
            admin: ADMIN.to_string(),
        };
        let permission_module_addr = app
            .instantiate_contract(
                permission_code_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        permission_module_addr
    }

    mod module_permissions {
        use super::*;

        use crate::{
            msg::{ExecuteMsg, QueryMsg},
            ContractError,
        };

        use rift_types::{module::Modules, permission::Permissions};

        #[test]
        fn test_update_happy_path() {
            let mut app = mock_app();
            let permission_module_addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateModulePermissions {
                module: Modules::MintModule,
                permissions: vec![Permissions::Ownership],
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    permission_module_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let msg = QueryMsg::ModulePermissions(Modules::MintModule);
            let res: Vec<Permissions> = app
                .wrap()
                .query_wasm_smart(permission_module_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res, vec![Permissions::Ownership]);

            let msg = ExecuteMsg::UpdateModulePermissions {
                module: Modules::PermissionModule,
                permissions: vec![Permissions::Attribute, Permissions::Ownership],
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    permission_module_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let msg = QueryMsg::ModulePermissions(Modules::PermissionModule);
            let res: Vec<Permissions> = app
                .wrap()
                .query_wasm_smart(permission_module_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res, vec![Permissions::Attribute, Permissions::Ownership]);
        }

        #[test]
        fn test_update_unhappy_path() {
            let mut app = mock_app();
            let permission_module_addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateModulePermissions {
                module: Modules::MintModule,
                permissions: vec![Permissions::Ownership],
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(USER),
                    permission_module_addr.clone(),
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

    mod permission_check {
        use super::*;

        mod ownership_check {
            use super::*;

            use cosmwasm_std::to_binary;
            use rift_types::{collection::Collections, module::Modules, permission::Permissions};

            use crate::msg::{ExecuteMsg, OwnershipMsg, PermissionCheckMsg};

            // TODO: This is not possible to test here
            // Can only be done in controller?
            // #[test]
            // fn test_happy_path() {
            //     let mut app = mock_app();
            //     let permission_module_addr = proper_instantiate(&mut app);

            //     let msg = ExecuteMsg::UpdateModulePermissions {
            //         module: Modules::MintModule,
            //         permissions: vec![Permissions::Ownership],
            //     };
            //     let _ = app
            //         .execute_contract(
            //             Addr::unchecked(ADMIN),
            //             permission_module_addr.clone(),
            //             &msg,
            //             &vec![],
            //         )
            //         .unwrap();

            //     let permission_msg = to_binary(&vec![PermissionCheckMsg {
            //         permission_type: Permissions::Ownership,
            //         data: to_binary(&vec![OwnershipMsg {
            //             collection_type: Collections::Normal,
            //             collection_id: 1,
            //             token_id: 1,
            //             owner: USER.to_string(),
            //         }])
            //         .unwrap(),
            //     }])
            //     .unwrap();
            //     let _ = app
            //         .execute_contract(
            //             Addr::unchecked(ADMIN),
            //             permission_module_addr.clone(),
            //             &ExecuteMsg::Check {
            //                 module: Modules::MintModule,
            //                 msg: permission_msg,
            //             },
            //             &vec![],
            //         )
            //         .unwrap();
            // }
        }
    }
}
