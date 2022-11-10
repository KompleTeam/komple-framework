use cosmwasm_std::Decimal;
use cosmwasm_std::{Binary, to_binary};
use cosmwasm_std::{Addr, Empty, Uint128};
use cw20::Cw20Coin;
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_fee_module::msg::CustomPaymentAddress;
use komple_fee_module::msg::ExecuteMsg;
use komple_types::shared::RegisterMsg;
use komple_types::modules::Modules;
use komple_utils::funds::FundsError;
use std::str::FromStr;
use komple_types::modules::fee::{Fees, FixedPayment, PercentagePayment};

const ADMIN: &str = "juno..admin";
const KOMPLE: &str = "juno..komple";
const COMMUNITY: &str = "juno..community";
const PAYMENT: &str = "juno..test";
const CW20_DENOM: &str = "cwdenom";

pub fn fee_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_fee_module::contract::execute,
        komple_fee_module::contract::instantiate,
        komple_fee_module::contract::query,
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

fn mock_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &Addr::unchecked(ADMIN), vec![])
            .unwrap();
    })
}

fn setup_fee_contract(app: &mut App) -> Addr {
    let code_id = app.store_code(fee_contract());

    let msg = RegisterMsg {
        admin: ADMIN.to_string(),
        data: None,
    };

    app.instantiate_contract(code_id, Addr::unchecked(ADMIN), &msg, &[], "test", None)
        .unwrap()
}

fn setup_fee(
    app: &mut App,
    contract: Addr,
    fee_type: Fees,
    module_name: &str,
    fee_name: &str,
    data: Binary,
) {
    let msg = ExecuteMsg::SetFee {
        fee_type,
        module_name: module_name.to_string(),
        fee_name: fee_name.to_string(),
        data,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), contract, &msg, &[])
        .unwrap();
}

fn setup_cw20_token(app: &mut App) -> Addr {
    let code_id = app.store_code(cw20_contract());
    let msg = Cw20InstantiateMsg {
        name: "Test token".to_string(),
        symbol: CW20_DENOM.to_string(),
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: ADMIN.to_string(),
            amount: Uint128::new(2_000_000),
        }],
        mint: None,
        marketing: None,
    };
    app.instantiate_contract(code_id, Addr::unchecked(ADMIN), &msg, &[], "test", None)
        .unwrap()
}

mod distribute {
    use super::*;

    mod percentage {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let fee_module_addr = setup_fee_contract(&mut app);

            setup_fee(
                &mut app,
                fee_module_addr.clone(),
                Fees::Percentage,
                Modules::Marketplace.as_str(),
                "komple",
                to_binary(&PercentagePayment {
                    value: Decimal::from_str("0.04").unwrap(),
                    address: Some(KOMPLE.to_string()),
                })
                .unwrap(),
            );
            setup_fee(
                &mut app,
                fee_module_addr.clone(),
                Fees::Percentage,
                Modules::Marketplace.as_str(),
                "community",
                to_binary(&PercentagePayment {
                    value: Decimal::from_str("0.02").unwrap(),
                    address: Some(COMMUNITY.to_string()),
                })
                .unwrap(),
            );
            setup_fee(
                &mut app,
                fee_module_addr.clone(),
                Fees::Percentage,
                Modules::Marketplace.as_str(),
                "payment",
                to_binary(&PercentagePayment {
                    value: Decimal::from_str("0.03").unwrap(),
                    address: Some(PAYMENT.to_string()),
                })
                .unwrap(),
            );

            let cw20_addr = setup_cw20_token(&mut app);

            let msg = Cw20ExecuteMsg::Send {
                contract: fee_module_addr.to_string(),
                amount: Uint128::new(360_000),
                msg: to_binary(&ExecuteMsg::Distribute {
                    fee_type: Fees::Percentage,
                    module_name: Modules::Marketplace.to_string(),
                    custom_payment_addresses: None,
                })
                .unwrap(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), cw20_addr.clone(), &msg, &[])
                .unwrap();

            // 4_000_000 purchase price
            // 360_000 fee taken
            // 160_000 for komple
            // 120_000 for payment
            // 80_000 for community

            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    cw20_addr.clone(),
                    &Cw20QueryMsg::Balance {
                        address: KOMPLE.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(balance.balance, Uint128::new(160_000));
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    cw20_addr.clone(),
                    &Cw20QueryMsg::Balance {
                        address: COMMUNITY.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(balance.balance, Uint128::new(80_000));
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    cw20_addr.clone(),
                    &Cw20QueryMsg::Balance {
                        address: PAYMENT.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(balance.balance, Uint128::new(120_000));

            let msg = Cw20ExecuteMsg::Send {
                contract: fee_module_addr.to_string(),
                amount: Uint128::new(630_000),
                msg: to_binary(&ExecuteMsg::Distribute {
                    fee_type: Fees::Percentage,
                    module_name: Modules::Marketplace.to_string(),
                    custom_payment_addresses: None,
                })
                .unwrap(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), cw20_addr.clone(), &msg, &[])
                .unwrap();

            // 7_000_000 purchase price
            // 630_000 fee taken
            // 280_000 for komple
            // 140_000 for community
            // 210_000 for payment

            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    cw20_addr.clone(),
                    &Cw20QueryMsg::Balance {
                        address: KOMPLE.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(balance.balance, Uint128::new(440_000));
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    cw20_addr.clone(),
                    &Cw20QueryMsg::Balance {
                        address: COMMUNITY.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(balance.balance, Uint128::new(220_000));
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    cw20_addr.clone(),
                    &Cw20QueryMsg::Balance {
                        address: PAYMENT.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(balance.balance, Uint128::new(330_000));
        }

        #[test]
        fn test_custom_addresses() {
            let mut app = mock_app();
            let fee_module_addr = setup_fee_contract(&mut app);

            setup_fee(
                &mut app,
                fee_module_addr.clone(),
                Fees::Percentage,
                Modules::Marketplace.as_str(),
                "komple",
                to_binary(&PercentagePayment {
                    value: Decimal::from_str("0.04").unwrap(),
                    address: Some(KOMPLE.to_string()),
                })
                .unwrap(),
            );
            setup_fee(
                &mut app,
                fee_module_addr.clone(),
                Fees::Percentage,
                Modules::Marketplace.as_str(),
                "community",
                to_binary(&PercentagePayment {
                    value: Decimal::from_str("0.02").unwrap(),
                    address: Some(COMMUNITY.to_string()),
                })
                .unwrap(),
            );
            setup_fee(
                &mut app,
                fee_module_addr.clone(),
                Fees::Percentage,
                Modules::Marketplace.as_str(),
                "payment",
                to_binary(&PercentagePayment {
                    value: Decimal::from_str("0.03").unwrap(),
                    address: Some(PAYMENT.to_string()),
                })
                .unwrap(),
            );

            let cw20_addr = setup_cw20_token(&mut app);

            const NEW_COMMUNITY: &str = "new_community";
            const NEW_PAYMENT: &str = "new_payment";

            let msg = Cw20ExecuteMsg::Send {
                contract: fee_module_addr.to_string(),
                amount: Uint128::new(360_000),
                msg: to_binary(&ExecuteMsg::Distribute {
                    fee_type: Fees::Percentage,
                    module_name: Modules::Marketplace.to_string(),
                    custom_payment_addresses: Some(vec![
                        CustomPaymentAddress {
                            fee_name: "community".to_string(),
                            address: NEW_COMMUNITY.to_string(),
                        },
                        CustomPaymentAddress {
                            fee_name: "payment".to_string(),
                            address: NEW_PAYMENT.to_string(),
                        },
                    ]),
                })
                .unwrap(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), cw20_addr.clone(), &msg, &[])
                .unwrap();

            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    cw20_addr.clone(),
                    &Cw20QueryMsg::Balance {
                        address: COMMUNITY.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(balance.balance, Uint128::new(0));
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    cw20_addr.clone(),
                    &Cw20QueryMsg::Balance {
                        address: NEW_COMMUNITY.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(balance.balance, Uint128::new(80_000));
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    cw20_addr.clone(),
                    &Cw20QueryMsg::Balance {
                        address: PAYMENT.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(balance.balance, Uint128::new(0));
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    cw20_addr.clone(),
                    &Cw20QueryMsg::Balance {
                        address: NEW_PAYMENT.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(balance.balance, Uint128::new(120_000));
        }
    }

    mod fixed {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let fee_module_addr = setup_fee_contract(&mut app);

            setup_fee(
                &mut app,
                fee_module_addr.clone(),
                Fees::Fixed,
                Modules::Hub.as_str(),
                "creation",
                to_binary(&FixedPayment {
                    value: Uint128::new(1_000_000),
                    address: Some(COMMUNITY.to_string()),
                })
                .unwrap(),
            );
            setup_fee(
                &mut app,
                fee_module_addr.clone(),
                Fees::Fixed,
                Modules::Hub.as_str(),
                "module_register",
                to_binary(&FixedPayment {
                    value: Uint128::new(500_000),
                    address: Some(COMMUNITY.to_string()),
                })
                .unwrap(),
            );
            setup_fee(
                &mut app,
                fee_module_addr.clone(),
                Fees::Fixed,
                Modules::Hub.as_str(),
                "random",
                to_binary(&FixedPayment {
                    value: Uint128::new(250_000),
                    address: Some(PAYMENT.to_string()),
                })
                .unwrap(),
            );

            let cw20_addr = setup_cw20_token(&mut app);

            let msg = Cw20ExecuteMsg::Send {
                contract: fee_module_addr.to_string(),
                amount: Uint128::new(1_750_000),
                msg: to_binary(&ExecuteMsg::Distribute {
                    fee_type: Fees::Fixed,
                    module_name: Modules::Hub.to_string(),
                    custom_payment_addresses: None,
                })
                .unwrap(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), cw20_addr.clone(), &msg, &[])
                .unwrap();

            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    cw20_addr.clone(),
                    &Cw20QueryMsg::Balance {
                        address: COMMUNITY.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(balance.balance, Uint128::new(1_500_000));
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    cw20_addr.clone(),
                    &Cw20QueryMsg::Balance {
                        address: PAYMENT.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(balance.balance, Uint128::new(250_000));
        }

        #[test]
        fn test_custom_addresses() {
            let mut app = mock_app();
            let fee_module_addr = setup_fee_contract(&mut app);

            setup_fee(
                &mut app,
                fee_module_addr.clone(),
                Fees::Fixed,
                Modules::Hub.as_str(),
                "creation",
                to_binary(&FixedPayment {
                    value: Uint128::new(1_000_000),
                    address: Some(COMMUNITY.to_string()),
                })
                .unwrap(),
            );
            setup_fee(
                &mut app,
                fee_module_addr.clone(),
                Fees::Fixed,
                Modules::Hub.as_str(),
                "module_register",
                to_binary(&FixedPayment {
                    value: Uint128::new(500_000),
                    address: Some(COMMUNITY.to_string()),
                })
                .unwrap(),
            );
            setup_fee(
                &mut app,
                fee_module_addr.clone(),
                Fees::Fixed,
                Modules::Hub.as_str(),
                "random",
                to_binary(&FixedPayment {
                    value: Uint128::new(250_000),
                    address: Some(PAYMENT.to_string()),
                })
                .unwrap(),
            );

            const NEW_COMMUNITY: &str = "new_community";
            const NEW_PAYMENT: &str = "new_payment";

            let cw20_addr = setup_cw20_token(&mut app);

            let msg = Cw20ExecuteMsg::Send {
                contract: fee_module_addr.to_string(),
                amount: Uint128::new(1_750_000),
                msg: to_binary(&ExecuteMsg::Distribute {
                    fee_type: Fees::Fixed,
                    module_name: Modules::Hub.to_string(),
                    custom_payment_addresses: Some(vec![
                        CustomPaymentAddress {
                            fee_name: "creation".to_string(),
                            address: NEW_COMMUNITY.to_string(),
                        },
                        CustomPaymentAddress {
                            fee_name: "module_register".to_string(),
                            address: NEW_COMMUNITY.to_string(),
                        },
                        CustomPaymentAddress {
                            fee_name: "random".to_string(),
                            address: NEW_PAYMENT.to_string(),
                        },
                    ]),
                })
                .unwrap(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), cw20_addr.clone(), &msg, &[])
                .unwrap();

            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    cw20_addr.clone(),
                    &Cw20QueryMsg::Balance {
                        address: NEW_COMMUNITY.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(balance.balance, Uint128::new(1_500_000));
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    cw20_addr.clone(),
                    &Cw20QueryMsg::Balance {
                        address: NEW_PAYMENT.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(balance.balance, Uint128::new(250_000));
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    cw20_addr.clone(),
                    &Cw20QueryMsg::Balance {
                        address: COMMUNITY.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(balance.balance, Uint128::new(0));
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    cw20_addr.clone(),
                    &Cw20QueryMsg::Balance {
                        address: PAYMENT.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(balance.balance, Uint128::new(0));
        }

        #[test]
        fn test_invalid_funds() {
            let mut app = mock_app();
            let fee_module_addr = setup_fee_contract(&mut app);

            setup_fee(
                &mut app,
                fee_module_addr.clone(),
                Fees::Fixed,
                Modules::Hub.as_str(),
                "creation",
                to_binary(&FixedPayment {
                    value: Uint128::new(1_000_000),
                    address: Some(COMMUNITY.to_string()),
                })
                .unwrap(),
            );
            setup_fee(
                &mut app,
                fee_module_addr.clone(),
                Fees::Fixed,
                Modules::Hub.as_str(),
                "module_register",
                to_binary(&FixedPayment {
                    value: Uint128::new(500_000),
                    address: Some(COMMUNITY.to_string()),
                })
                .unwrap(),
            );
            setup_fee(
                &mut app,
                fee_module_addr.clone(),
                Fees::Fixed,
                Modules::Hub.as_str(),
                "random",
                to_binary(&FixedPayment {
                    value: Uint128::new(250_000),
                    address: Some(PAYMENT.to_string()),
                })
                .unwrap(),
            );

            let cw20_addr = setup_cw20_token(&mut app);

            let msg = Cw20ExecuteMsg::Send {
                contract: fee_module_addr.to_string(),
                amount: Uint128::new(1_000_000),
                msg: to_binary(&ExecuteMsg::Distribute {
                    fee_type: Fees::Fixed,
                    module_name: Modules::Hub.to_string(),
                    custom_payment_addresses: None,
                })
                .unwrap(),
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), cw20_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().source().unwrap().to_string(),
                FundsError::InvalidFunds {
                    got: "1000000".to_string(),
                    expected: "1750000".to_string()
                }
                .to_string()
            );
        }
    }
}
