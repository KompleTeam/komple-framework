use cosmwasm_std::{coin, to_binary, StdError};
use cosmwasm_std::{Addr, Coin, Empty, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_fee_module::msg::InstantiateMsg as FeeModuleInstantiateMsg;
use komple_hub_module::{
    msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{HubInfo, WebsiteConfig},
    ContractError,
};
use komple_marketplace_module::msg::InstantiateMsg as MarketplaceModuleInstantiateMsg;
use komple_merge_module::msg::InstantiateMsg as MergeModuleInstantiateMsg;
use komple_mint_module::msg::InstantiateMsg as MintModuleInstantiateMsg;
use komple_permission_module::msg::InstantiateMsg as PermissionModuleInstantiateMsg;
use komple_types::{module::Modules, query::ResponseWrapper};

pub fn hub_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_hub_module::contract::execute,
        komple_hub_module::contract::instantiate,
        komple_hub_module::contract::query,
    )
    .with_reply(komple_hub_module::contract::reply);
    Box::new(contract)
}

pub fn mint_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_mint_module::contract::execute,
        komple_mint_module::contract::instantiate,
        komple_mint_module::contract::query,
    )
    .with_reply(komple_mint_module::contract::reply);
    Box::new(contract)
}

pub fn permission_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_permission_module::contract::execute,
        komple_permission_module::contract::instantiate,
        komple_permission_module::contract::query,
    );
    Box::new(contract)
}

pub fn merge_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_merge_module::contract::execute,
        komple_merge_module::contract::instantiate,
        komple_merge_module::contract::query,
    );
    Box::new(contract)
}

pub fn marketplace_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_marketplace_module::contract::execute,
        komple_marketplace_module::contract::instantiate,
        komple_marketplace_module::contract::query,
    );
    Box::new(contract)
}

pub fn fee_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_fee_module::contract::execute,
        komple_fee_module::contract::instantiate,
        komple_fee_module::contract::query,
    );
    Box::new(contract)
}

const USER: &str = "juno1shfqtuup76mngspx29gcquykjvvlx9na4kymlm";
const ADMIN: &str = "juno1qamfln8u5w8d3vlhp5t9mhmylfkgad4jz6t7cv";
const NATIVE_DENOM: &str = "native_denom";

fn mock_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(USER),
                vec![Coin {
                    denom: "some_denom".to_string(),
                    amount: Uint128::new(1_000_000),
                }],
            )
            .unwrap();
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
    let hub_code_id = app.store_code(hub_module());

    let msg = InstantiateMsg {
        admin: None,
        hub_info: HubInfo {
            name: "Test Hub".to_string(),
            description: "Test Hub".to_string(),
            image: "https://image.com".to_string(),
            external_link: None,
        },
        marbu_fee_module: None,
    };

    app.instantiate_contract(
        hub_code_id,
        Addr::unchecked(ADMIN),
        &msg,
        &[coin(1_000_000, NATIVE_DENOM)],
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
        let hub_code_id = app.store_code(hub_module());

        let msg = InstantiateMsg {
            admin: None,
            hub_info: HubInfo {
                name: "Test Hub".to_string(),
                description: "Test Hub".to_string(),
                image: "https://image.com".to_string(),
                external_link: None,
            },
            marbu_fee_module: None,
        };
        let _ = app
            .instantiate_contract(hub_code_id, Addr::unchecked(ADMIN), &msg, &[], "test", None)
            .unwrap();
    }

    #[test]
    fn invalid_description() {
        let mut app = mock_app();
        let hub_code_id = app.store_code(hub_module());

        let msg = InstantiateMsg {
            admin: None,
            hub_info: HubInfo {
                name: "Test Hub".to_string(),
                description: "Test HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest HubTest Hub".to_string(),
                image: "https://image.com".to_string(),
                external_link: None,
            },
            marbu_fee_module: None
        };

        let err = app
            .instantiate_contract(
                hub_code_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[coin(1_000_000, NATIVE_DENOM)],
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

    mod register_module {
        use super::*;

        #[test]
        fn test_register_mint_happy_path() {
            let mut app = mock_app();
            let hub_module_addr = proper_instantiate(&mut app);
            let mint_module_code_id = app.store_code(mint_module());

            let instantiate_msg = to_binary(&MintModuleInstantiateMsg {
                admin: ADMIN.to_string(),
            })
            .unwrap();
            let msg = ExecuteMsg::RegisterModule {
                module: Modules::Mint.to_string(),
                msg: instantiate_msg,
                code_id: mint_module_code_id,
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), hub_module_addr.clone(), &msg, &[])
                .unwrap();

            let msg = QueryMsg::ModuleAddress {
                module: Modules::Mint.to_string(),
            };
            let res: ResponseWrapper<String> =
                app.wrap().query_wasm_smart(hub_module_addr, &msg).unwrap();
            assert_eq!(res.data, "contract1")
        }

        #[test]
        fn test_register_permission_happy_path() {
            let mut app = mock_app();
            let hub_module_addr = proper_instantiate(&mut app);
            let permission_module_code_id = app.store_code(permission_module());

            let instantiate_msg = to_binary(&PermissionModuleInstantiateMsg {
                admin: ADMIN.to_string(),
            })
            .unwrap();
            let msg = ExecuteMsg::RegisterModule {
                module: Modules::Permission.to_string(),
                msg: instantiate_msg,
                code_id: permission_module_code_id,
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), hub_module_addr.clone(), &msg, &[])
                .unwrap();

            let msg = QueryMsg::ModuleAddress {
                module: Modules::Permission.to_string(),
            };
            let res: ResponseWrapper<String> =
                app.wrap().query_wasm_smart(hub_module_addr, &msg).unwrap();
            assert_eq!(res.data, "contract1")
        }

        #[test]
        fn test_register_merge_happy_path() {
            let mut app = mock_app();
            let hub_module_addr = proper_instantiate(&mut app);
            let merge_module_code_id = app.store_code(merge_module());

            let instantiate_msg = to_binary(&MergeModuleInstantiateMsg {
                admin: ADMIN.to_string(),
            })
            .unwrap();
            let msg = ExecuteMsg::RegisterModule {
                module: Modules::Merge.to_string(),
                msg: instantiate_msg,
                code_id: merge_module_code_id,
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), hub_module_addr.clone(), &msg, &[])
                .unwrap();

            let msg = QueryMsg::ModuleAddress {
                module: Modules::Merge.to_string(),
            };
            let res: ResponseWrapper<String> =
                app.wrap().query_wasm_smart(hub_module_addr, &msg).unwrap();
            assert_eq!(res.data, "contract1")
        }

        #[test]
        fn test_register_marketplace_happy_path() {
            let mut app = mock_app();
            let hub_module_addr = proper_instantiate(&mut app);
            let marketplace_module_code_id = app.store_code(marketplace_module());

            let instantiate_msg = to_binary(&MarketplaceModuleInstantiateMsg {
                admin: ADMIN.to_string(),
                native_denom: NATIVE_DENOM.to_string(),
            })
            .unwrap();
            let msg = ExecuteMsg::RegisterModule {
                module: Modules::Marketplace.to_string(),
                msg: instantiate_msg,
                code_id: marketplace_module_code_id,
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), hub_module_addr.clone(), &msg, &[])
                .unwrap();

            let msg = QueryMsg::ModuleAddress {
                module: Modules::Marketplace.to_string(),
            };
            let res: ResponseWrapper<String> =
                app.wrap().query_wasm_smart(hub_module_addr, &msg).unwrap();
            assert_eq!(res.data, "contract1")
        }

        #[test]
        fn test_register_fee_happy_path() {
            let mut app = mock_app();
            let hub_module_addr = proper_instantiate(&mut app);
            let fee_module_code_id = app.store_code(fee_module());

            let instantiate_msg = to_binary(&FeeModuleInstantiateMsg {
                admin: ADMIN.to_string(),
            })
            .unwrap();
            let msg = ExecuteMsg::RegisterModule {
                module: Modules::Fee.to_string(),
                msg: instantiate_msg,
                code_id: fee_module_code_id,
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), hub_module_addr.clone(), &msg, &[])
                .unwrap();

            let msg = QueryMsg::ModuleAddress {
                module: Modules::Fee.to_string(),
            };
            let res: ResponseWrapper<String> =
                app.wrap().query_wasm_smart(hub_module_addr, &msg).unwrap();
            assert_eq!(res.data, "contract1")
        }

        #[test]
        fn test_register_unhappy_path() {
            let mut app = mock_app();
            let hub_module_addr = proper_instantiate(&mut app);
            let mint_module_code_id = app.store_code(mint_module());

            let instantiate_msg = to_binary(&MintModuleInstantiateMsg {
                admin: ADMIN.to_string(),
            })
            .unwrap();
            let msg = ExecuteMsg::RegisterModule {
                module: Modules::Mint.to_string(),
                msg: instantiate_msg,
                code_id: mint_module_code_id,
            };
            let err = app
                .execute_contract(Addr::unchecked(USER), hub_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            )
        }
    }

    mod remove_native_modules {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let hub_module_addr = proper_instantiate(&mut app);
            let mint_module_code_id = app.store_code(mint_module());

            let instantiate_msg = to_binary(&MintModuleInstantiateMsg {
                admin: ADMIN.to_string(),
            })
            .unwrap();
            let msg = ExecuteMsg::RegisterModule {
                module: Modules::Mint.to_string(),
                msg: instantiate_msg,
                code_id: mint_module_code_id,
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), hub_module_addr.clone(), &msg, &[])
                .unwrap();

            let msg = QueryMsg::ModuleAddress {
                module: Modules::Mint.to_string(),
            };
            let res: ResponseWrapper<String> = app
                .wrap()
                .query_wasm_smart(hub_module_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data, "contract1");

            let msg = ExecuteMsg::DeregisterModule {
                module: Modules::Mint.to_string(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), hub_module_addr.clone(), &msg, &[])
                .unwrap();

            let msg = QueryMsg::ModuleAddress {
                module: Modules::Mint.to_string(),
            };
            let res: Result<ResponseWrapper<String>, StdError> =
                app.wrap().query_wasm_smart(hub_module_addr, &msg);
            assert!(res.is_err());
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let hub_module_addr = proper_instantiate(&mut app);
            let mint_module_code_id = app.store_code(mint_module());

            let instantiate_msg = to_binary(&MintModuleInstantiateMsg {
                admin: ADMIN.to_string(),
            })
            .unwrap();
            let msg = ExecuteMsg::RegisterModule {
                module: Modules::Mint.to_string(),
                msg: instantiate_msg,
                code_id: mint_module_code_id,
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), hub_module_addr.clone(), &msg, &[])
                .unwrap();

            let msg = ExecuteMsg::DeregisterModule {
                module: Modules::Mint.to_string(),
            };
            let err = app
                .execute_contract(Addr::unchecked(USER), hub_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }

        #[test]
        fn test_invalid_module() {
            let mut app = mock_app();
            let hub_module_addr = proper_instantiate(&mut app);
            let mint_module_code_id = app.store_code(mint_module());

            let instantiate_msg = to_binary(&MintModuleInstantiateMsg {
                admin: ADMIN.to_string(),
            })
            .unwrap();
            let msg = ExecuteMsg::RegisterModule {
                module: Modules::Mint.to_string(),
                msg: instantiate_msg,
                code_id: mint_module_code_id,
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), hub_module_addr.clone(), &msg, &[])
                .unwrap();

            let msg = ExecuteMsg::DeregisterModule {
                module: Modules::Swap.to_string(),
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), hub_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::InvalidModule {}.to_string()
            );
        }
    }

    mod update_website_config {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let hub_module_addr = proper_instantiate(&mut app);

            let msg = QueryMsg::Config {};
            let res: ResponseWrapper<ConfigResponse> = app
                .wrap()
                .query_wasm_smart(hub_module_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.website_config, None);

            let msg = ExecuteMsg::UpdateWebsiteConfig {
                background_color: Some("#00FFEE".to_string()),
                background_image: Some("ifps://some-image".to_string()),
                banner_image: Some("ipfs://some-banner".to_string()),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), hub_module_addr.clone(), &msg, &[])
                .unwrap();

            let msg = QueryMsg::Config {};
            let res: ResponseWrapper<ConfigResponse> =
                app.wrap().query_wasm_smart(hub_module_addr, &msg).unwrap();
            assert_eq!(
                res.data.website_config,
                Some(WebsiteConfig {
                    background_color: Some("#00FFEE".to_string()),
                    background_image: Some("ifps://some-image".to_string()),
                    banner_image: Some("ipfs://some-banner".to_string()),
                })
            )
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let hub_module_addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateWebsiteConfig {
                background_color: Some("#00FFEE".to_string()),
                background_image: Some("ifps://some-image".to_string()),
                banner_image: Some("ipfs://some-banner".to_string()),
            };
            let err = app
                .execute_contract(Addr::unchecked(USER), hub_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            )
        }
    }

    mod update_hub_info {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let hub_module_addr = proper_instantiate(&mut app);

            let msg = QueryMsg::Config {};
            let res: ResponseWrapper<ConfigResponse> = app
                .wrap()
                .query_wasm_smart(hub_module_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.website_config, None);

            let msg = ExecuteMsg::UpdateHubInfo {
                name: "New Name".to_string(),
                description: "New Description".to_string(),
                image: "https://new-image.com".to_string(),
                external_link: Some("https://some-link".to_string()),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), hub_module_addr.clone(), &msg, &[])
                .unwrap();

            let msg = QueryMsg::Config {};
            let res: ResponseWrapper<ConfigResponse> =
                app.wrap().query_wasm_smart(hub_module_addr, &msg).unwrap();
            assert_eq!(
                res.data.hub_info,
                HubInfo {
                    name: "New Name".to_string(),
                    description: "New Description".to_string(),
                    image: "https://new-image.com".to_string(),
                    external_link: Some("https://some-link".to_string()),
                }
            )
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let hub_module_addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateHubInfo {
                name: "New Name".to_string(),
                description: "New Description".to_string(),
                image: "https://new-image.com".to_string(),
                external_link: Some("https://some-link".to_string()),
            };
            let err = app
                .execute_contract(Addr::unchecked(USER), hub_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            )
        }
    }

    mod update_operators {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let hub_module_addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateOperators {
                addrs: vec![
                    "juno..first".to_string(),
                    "juno..second".to_string(),
                    "juno..first".to_string(),
                ],
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), hub_module_addr.clone(), &msg, &[])
                .unwrap();

            let msg = QueryMsg::Operators {};
            let res: ResponseWrapper<Vec<String>> = app
                .wrap()
                .query_wasm_smart(hub_module_addr.clone(), &msg)
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
                    hub_module_addr.clone(),
                    &msg,
                    &[],
                )
                .unwrap();

            let msg = QueryMsg::Operators {};
            let res: ResponseWrapper<Vec<String>> =
                app.wrap().query_wasm_smart(hub_module_addr, &msg).unwrap();
            assert_eq!(res.data.len(), 1);
            assert_eq!(res.data[0], "juno..third");
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let hub_module_addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateOperators {
                addrs: vec!["juno..first".to_string(), "juno..second".to_string()],
            };
            let err = app
                .execute_contract(Addr::unchecked(USER), hub_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }

        #[test]
        fn test_invalid_operator() {
            let mut app = mock_app();
            let hub_module_addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateOperators {
                addrs: vec!["juno..first".to_string(), "juno..second".to_string()],
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), hub_module_addr.clone(), &msg, &[])
                .unwrap();

            let err = app
                .execute_contract(Addr::unchecked("juno..third"), hub_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }
    }
}