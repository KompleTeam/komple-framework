use crate::{
    msg::{ExecuteMsg, InstantiateMsg, MarketplaceFundInfo, QueryMsg},
    ContractError,
};
use cosmwasm_std::{to_binary, Addr, Coin, Empty, Uint128};
use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
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

pub fn cw20_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

const USER: &str = "juno..user";
const ADMIN: &str = "juno..admin";
const NATIVE_DENOM: &str = "native_denom";
const CW20_DENOM: &str = "cwdenom";

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
                fund_info: MarketplaceFundInfo {
                    is_native: true,
                    denom: NATIVE_DENOM.to_string(),
                    cw20_address: None,
                },
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

fn setup_cw20_token(app: &mut App) -> Addr {
    let cw20_code_id = app.store_code(cw20_contract());
    let msg = Cw20InstantiateMsg {
        name: "test".to_string(),
        symbol: CW20_DENOM.to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: None,
        marketing: None,
    };
    app.instantiate_contract(
        cw20_code_id,
        Addr::unchecked(ADMIN),
        &msg,
        &[],
        "test",
        None,
    )
    .unwrap()
}

mod instantiate {
    use komple_utils::funds::FundsError;

    use super::*;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let marketplace_code_id = app.store_code(marketplace_module());

        let msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(
                to_binary(&InstantiateMsg {
                    fund_info: MarketplaceFundInfo {
                        is_native: true,
                        denom: NATIVE_DENOM.to_string(),
                        cw20_address: None,
                    },
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
        .unwrap();

        // Cw20 support
        let cw20_addr = setup_cw20_token(&mut app);
        let msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(
                to_binary(&InstantiateMsg {
                    fund_info: MarketplaceFundInfo {
                        is_native: false,
                        denom: CW20_DENOM.to_string(),
                        cw20_address: Some(cw20_addr.to_string()),
                    },
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
        .unwrap();
    }

    #[test]
    fn test_invalid_cw20_token() {
        let mut app = mock_app();
        let marketplace_code_id = app.store_code(marketplace_module());

        // Cw20 support
        let cw20_addr = setup_cw20_token(&mut app);

        let msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(
                to_binary(&InstantiateMsg {
                    fund_info: MarketplaceFundInfo {
                        is_native: false,
                        denom: "invalid".to_string(),
                        cw20_address: Some(cw20_addr.to_string()),
                    },
                })
                .unwrap(),
            ),
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
            FundsError::InvalidCw20Token {}.to_string()
        );

        let msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(
                to_binary(&InstantiateMsg {
                    fund_info: MarketplaceFundInfo {
                        is_native: false,
                        denom: CW20_DENOM.to_string(),
                        cw20_address: None,
                    },
                })
                .unwrap(),
            ),
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
            FundsError::InvalidCw20Token {}.to_string()
        );
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
        use komple_types::shared::query::ResponseWrapper;

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

    mod update_buy_lock {
        use komple_types::modules::marketplace::Listing;
        use komple_types::shared::query::ResponseWrapper;

        use crate::state::Config;

        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateBuyLock { lock: true };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &vec![])
                .unwrap();

            let res: ResponseWrapper<Config> = app
                .wrap()
                .query_wasm_smart(addr.clone(), &QueryMsg::Config {})
                .unwrap();
            assert_eq!(res.data.buy_lock, true);

            let msg = ExecuteMsg::Buy {
                listing_type: Listing::Fixed,
                collection_id: 1,
                token_id: 1,
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::BuyLocked {}.to_string()
            );
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateBuyLock { lock: true };
            let err = app
                .execute_contract(Addr::unchecked(USER), addr.clone(), &msg, &vec![])
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
