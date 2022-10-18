use crate::{
    msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{HubInfo, WebsiteConfig},
    ContractError,
};
use cosmwasm_std::coin;
use cosmwasm_std::{Addr, Coin, Empty, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_types::query::ResponseWrapper;

pub fn hub_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(crate::contract::reply);
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
        use crate::state::HubInfo;

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
