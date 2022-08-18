use crate::{
    msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg},
    state::WebsiteConfig,
    ContractError,
};
use cosmwasm_std::{Addr, Coin, Empty, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_fee_contract::msg::InstantiateMsg as FeeContractInstantiateMsg;
use komple_types::{module::Modules, query::ResponseWrapper};

pub fn collection_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(crate::contract::reply);
    Box::new(contract)
}

pub fn mint_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        mint_module::contract::execute,
        mint_module::contract::instantiate,
        mint_module::contract::query,
    )
    .with_reply(mint_module::contract::reply);
    Box::new(contract)
}

pub fn permission_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        permission_module::contract::execute,
        permission_module::contract::instantiate,
        permission_module::contract::query,
    );
    Box::new(contract)
}

pub fn merge_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        merge_module::contract::execute,
        merge_module::contract::instantiate,
        merge_module::contract::query,
    );
    Box::new(contract)
}

pub fn marketplace_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        marketplace_module::contract::execute,
        marketplace_module::contract::instantiate,
        marketplace_module::contract::query,
    );
    Box::new(contract)
}

pub fn fee_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_fee_contract::contract::execute,
        komple_fee_contract::contract::instantiate,
        komple_fee_contract::contract::query,
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

fn setup_fee_contract(app: &mut App) -> Addr {
    let fee_code_id = app.store_code(fee_contract());

    let msg = FeeContractInstantiateMsg {
        komple_address: ADMIN.to_string(),
        payment_address: "juno..community".to_string(),
    };
    let fee_contract_addr = app
        .instantiate_contract(
            fee_code_id,
            Addr::unchecked(ADMIN),
            &msg,
            &vec![],
            "test",
            None,
        )
        .unwrap();

    fee_contract_addr
}

fn proper_instantiate(app: &mut App) -> Addr {
    let collection_code_id = app.store_code(collection_contract());

    let msg = InstantiateMsg {
        name: "Test Collection".to_string(),
        description: "Test Collection".to_string(),
        image: "https://image.com".to_string(),
        external_link: None,
    };
    let collection_contract_addr = app
        .instantiate_contract(
            collection_code_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "test",
            None,
        )
        .unwrap();

    collection_contract_addr
}

mod actions {
    use super::*;

    mod init_modules {
        use super::*;

        mod mint_module_tests {
            use super::*;

            #[test]
            fn test_init_happy_path() {
                let mut app = mock_app();
                let collection_contract_addr = proper_instantiate(&mut app);
                let mint_module_code_id = app.store_code(mint_module());

                let msg = ExecuteMsg::InitMintModule {
                    code_id: mint_module_code_id,
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        collection_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = QueryMsg::ModuleAddress(Modules::MintModule);
                let res: ResponseWrapper<String> = app
                    .wrap()
                    .query_wasm_smart(collection_contract_addr, &msg)
                    .unwrap();
                assert_eq!(res.data, "contract1")
            }

            #[test]
            fn test_init_unhappy_path() {
                let mut app = mock_app();
                let collection_contract_addr = proper_instantiate(&mut app);
                let mint_module_code_id = app.store_code(mint_module());

                let msg = ExecuteMsg::InitMintModule {
                    code_id: mint_module_code_id,
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        collection_contract_addr.clone(),
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

            #[test]
            fn test_init_module() {
                let mut app = mock_app();
                let collection_contract_addr = proper_instantiate(&mut app);
                let permission_module_code_id = app.store_code(permission_module());

                let msg = ExecuteMsg::InitPermissionModule {
                    code_id: permission_module_code_id,
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        collection_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = QueryMsg::ModuleAddress(Modules::PermissionModule);
                let res: ResponseWrapper<String> = app
                    .wrap()
                    .query_wasm_smart(collection_contract_addr, &msg)
                    .unwrap();
                assert_eq!(res.data, "contract1")
            }

            #[test]
            fn test_init_unhappy_path() {
                let mut app = mock_app();
                let collection_contract_addr = proper_instantiate(&mut app);
                let permission_module_code_id = app.store_code(permission_module());

                let msg = ExecuteMsg::InitPermissionModule {
                    code_id: permission_module_code_id,
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        collection_contract_addr.clone(),
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
        mod merge_module_tests {
            use super::*;

            #[test]
            fn test_init_module() {
                let mut app = mock_app();
                let collection_contract_addr = proper_instantiate(&mut app);
                let merge_module_code_id = app.store_code(merge_module());

                let msg = ExecuteMsg::InitMergeModule {
                    code_id: merge_module_code_id,
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        collection_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = QueryMsg::ModuleAddress(Modules::MergeModule);
                let res: ResponseWrapper<String> = app
                    .wrap()
                    .query_wasm_smart(collection_contract_addr, &msg)
                    .unwrap();
                assert_eq!(res.data, "contract1")
            }

            #[test]
            fn test_init_unhappy_path() {
                let mut app = mock_app();
                let collection_contract_addr = proper_instantiate(&mut app);
                let merge_module_code_id = app.store_code(merge_module());

                let msg = ExecuteMsg::InitMergeModule {
                    code_id: merge_module_code_id,
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        collection_contract_addr.clone(),
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

        mod marketplace_module_tests {
            use super::*;

            #[test]
            fn test_init_module() {
                let mut app = mock_app();
                setup_fee_contract(&mut app);
                let collection_contract_addr = proper_instantiate(&mut app);
                let marketplace_module_code_id = app.store_code(marketplace_module());

                let msg = ExecuteMsg::InitMarketplaceModule {
                    code_id: marketplace_module_code_id,
                    native_denom: NATIVE_DENOM.to_string(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        collection_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = QueryMsg::ModuleAddress(Modules::MarketplaceModule);
                let res: ResponseWrapper<String> = app
                    .wrap()
                    .query_wasm_smart(collection_contract_addr, &msg)
                    .unwrap();
                assert_eq!(res.data, "contract2")
            }

            #[test]
            fn test_init_unhappy_path() {
                let mut app = mock_app();
                let collection_contract_addr = proper_instantiate(&mut app);
                let marketplace_module_code_id = app.store_code(marketplace_module());

                let msg = ExecuteMsg::InitMarketplaceModule {
                    code_id: marketplace_module_code_id,
                    native_denom: NATIVE_DENOM.to_string(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        collection_contract_addr.clone(),
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

    mod update_website_config {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let collection_contract_addr = proper_instantiate(&mut app);

            let msg = QueryMsg::Config {};
            let res: ResponseWrapper<ConfigResponse> = app
                .wrap()
                .query_wasm_smart(collection_contract_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.website_config, None);

            let msg = ExecuteMsg::UpdateWebsiteConfig {
                background_color: Some("#00FFEE".to_string()),
                background_image: Some("ifps://some-image".to_string()),
                banner_image: Some("ipfs://some-banner".to_string()),
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    collection_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let msg = QueryMsg::Config {};
            let res: ResponseWrapper<ConfigResponse> = app
                .wrap()
                .query_wasm_smart(collection_contract_addr, &msg)
                .unwrap();
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
            let collection_contract_addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateWebsiteConfig {
                background_color: Some("#00FFEE".to_string()),
                background_image: Some("ifps://some-image".to_string()),
                banner_image: Some("ipfs://some-banner".to_string()),
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(USER),
                    collection_contract_addr.clone(),
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

    mod update_collection_info {
        use crate::state::CollectionInfo;

        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let collection_contract_addr = proper_instantiate(&mut app);

            let msg = QueryMsg::Config {};
            let res: ResponseWrapper<ConfigResponse> = app
                .wrap()
                .query_wasm_smart(collection_contract_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.website_config, None);

            let msg = ExecuteMsg::UpdateCollectionInfo {
                name: "New Name".to_string(),
                description: "New Description".to_string(),
                image: "https://new-image.com".to_string(),
                external_link: Some("https://some-link".to_string()),
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    collection_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let msg = QueryMsg::Config {};
            let res: ResponseWrapper<ConfigResponse> = app
                .wrap()
                .query_wasm_smart(collection_contract_addr, &msg)
                .unwrap();
            assert_eq!(
                res.data.collection_info,
                CollectionInfo {
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
            let collection_contract_addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateCollectionInfo {
                name: "New Name".to_string(),
                description: "New Description".to_string(),
                image: "https://new-image.com".to_string(),
                external_link: Some("https://some-link".to_string()),
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(USER),
                    collection_contract_addr.clone(),
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
