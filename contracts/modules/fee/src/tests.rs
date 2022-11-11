use crate::msg::{CustomPaymentAddress, FixedFeeResponse, PercentageFeeResponse};
use crate::msg::{ExecuteMsg, QueryMsg};
use crate::state::Config;
use crate::ContractError;
use cosmwasm_std::Decimal;
use cosmwasm_std::StdError;
use cosmwasm_std::{coin, Addr, Empty, Uint128};
use cosmwasm_std::{to_binary, Binary};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_types::modules::fee::{Fees, FixedPayment, PercentagePayment};
use komple_types::modules::Modules;
use komple_types::shared::query::ResponseWrapper;
use komple_types::shared::RegisterMsg;
use std::str::FromStr;

const ADMIN: &str = "juno..admin";
const KOMPLE: &str = "juno..komple";
const COMMUNITY: &str = "juno..community";
const PAYMENT: &str = "juno..test";
const NATIVE_DENOM: &str = "native_denom";

pub fn fee_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

fn mock_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(ADMIN),
                vec![coin(2_000_000, NATIVE_DENOM)],
            )
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

mod instantiation {
    use super::*;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let code_id = app.store_code(fee_contract());

        let msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: None,
        };
        let addr = app
            .instantiate_contract(code_id, Addr::unchecked(ADMIN), &msg, &[], "test", None)
            .unwrap();

        let msg = QueryMsg::Config {};
        let res: ResponseWrapper<Config> = app.wrap().query_wasm_smart(addr, &msg).unwrap();
        assert_eq!(res.data.admin, ADMIN.to_string());
    }
}

mod actions {
    use super::*;

    mod set_fee {
        use super::*;

        mod percentage {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let addr = setup_fee_contract(&mut app);

                let msg = ExecuteMsg::SetFee {
                    fee_type: Fees::Percentage,
                    module_name: Modules::Marketplace.to_string(),
                    fee_name: "komple".to_string(),
                    data: to_binary(&PercentagePayment {
                        address: Some(KOMPLE.to_string()),
                        value: Decimal::from_str("0.04").unwrap(),
                    })
                    .unwrap(),
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &[])
                    .unwrap();

                let msg = QueryMsg::PercentageFee {
                    module_name: Modules::Marketplace.to_string(),
                    fee_name: "komple".to_string(),
                };
                let res: ResponseWrapper<PercentageFeeResponse> =
                    app.wrap().query_wasm_smart(addr, &msg).unwrap();
                assert_eq!(res.data.module_name, Modules::Marketplace.as_str());
                assert_eq!(res.data.fee_name, "komple");
                assert_eq!(res.data.address, Some(KOMPLE.to_string()));
                assert_eq!(res.data.value, Decimal::from_str("0.04").unwrap());
            }

            #[test]
            fn test_invalid_admin() {
                let mut app = mock_app();
                let addr = setup_fee_contract(&mut app);

                let msg = ExecuteMsg::SetFee {
                    fee_type: Fees::Percentage,
                    module_name: Modules::Marketplace.to_string(),
                    fee_name: "komple".to_string(),
                    data: to_binary(&PercentagePayment {
                        address: Some(KOMPLE.to_string()),
                        value: Decimal::from_str("0.04").unwrap(),
                    })
                    .unwrap(),
                };
                let err = app
                    .execute_contract(Addr::unchecked(PAYMENT), addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::Unauthorized {}.to_string()
                );
            }

            #[test]
            fn test_invalid_fee_value() {
                let mut app = mock_app();
                let addr = setup_fee_contract(&mut app);

                let msg = ExecuteMsg::SetFee {
                    fee_type: Fees::Percentage,
                    module_name: Modules::Marketplace.to_string(),
                    fee_name: "komple".to_string(),
                    data: to_binary(&PercentagePayment {
                        address: Some(KOMPLE.to_string()),
                        value: Decimal::from_str("1.1").unwrap(),
                    })
                    .unwrap(),
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::InvalidFee {}.to_string()
                )
            }

            #[test]
            fn test_invalid_total_fee_value() {
                let mut app = mock_app();
                let addr = setup_fee_contract(&mut app);

                setup_fee(
                    &mut app,
                    addr.clone(),
                    Fees::Percentage,
                    Modules::Marketplace.as_str(),
                    "percentage_1",
                    to_binary(&PercentagePayment {
                        value: Decimal::from_str("0.4").unwrap(),
                        address: Some("address_1".to_string()),
                    })
                    .unwrap(),
                );
                setup_fee(
                    &mut app,
                    addr.clone(),
                    Fees::Percentage,
                    Modules::Marketplace.as_str(),
                    "percentage_2",
                    to_binary(&PercentagePayment {
                        value: Decimal::from_str("0.5").unwrap(),
                        address: Some("address_2".to_string()),
                    })
                    .unwrap(),
                );

                let msg = ExecuteMsg::SetFee {
                    fee_type: Fees::Percentage,
                    module_name: Modules::Marketplace.to_string(),
                    fee_name: "komple".to_string(),
                    data: to_binary(&PercentagePayment {
                        address: Some(KOMPLE.to_string()),
                        value: Decimal::from_str("0.2").unwrap(),
                    })
                    .unwrap(),
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::InvalidTotalFee {}.to_string()
                );

                let msg = ExecuteMsg::SetFee {
                    fee_type: Fees::Percentage,
                    module_name: Modules::Marketplace.to_string(),
                    fee_name: "komple".to_string(),
                    data: to_binary(&PercentagePayment {
                        address: Some(KOMPLE.to_string()),
                        value: Decimal::from_str("0.3").unwrap(),
                    })
                    .unwrap(),
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::InvalidTotalFee {}.to_string()
                )
            }
        }

        mod fixed {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let addr = setup_fee_contract(&mut app);

                let msg = ExecuteMsg::SetFee {
                    fee_type: Fees::Fixed,
                    module_name: Modules::Hub.to_string(),
                    fee_name: "creation".to_string(),
                    data: to_binary(&FixedPayment {
                        address: Some(COMMUNITY.to_string()),
                        value: Uint128::new(1_000_000),
                    })
                    .unwrap(),
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &[])
                    .unwrap();

                let msg = QueryMsg::FixedFee {
                    module_name: Modules::Hub.to_string(),
                    fee_name: "creation".to_string(),
                };
                let res: ResponseWrapper<FixedFeeResponse> =
                    app.wrap().query_wasm_smart(addr, &msg).unwrap();
                assert_eq!(res.data.module_name, Modules::Hub.as_str());
                assert_eq!(res.data.fee_name, "creation");
                assert_eq!(res.data.address, Some(COMMUNITY.to_string()));
                assert_eq!(res.data.value, Uint128::new(1_000_000));
            }

            #[test]
            fn test_invalid_admin() {
                let mut app = mock_app();
                let addr = setup_fee_contract(&mut app);

                let msg = ExecuteMsg::SetFee {
                    fee_type: Fees::Fixed,
                    module_name: Modules::Hub.to_string(),
                    fee_name: "creation".to_string(),
                    data: to_binary(&FixedPayment {
                        address: Some(COMMUNITY.to_string()),
                        value: Uint128::new(1_000_000),
                    })
                    .unwrap(),
                };
                let err = app
                    .execute_contract(Addr::unchecked(PAYMENT), addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::Unauthorized {}.to_string()
                );
            }

            #[test]
            fn test_invalid_fee_value() {
                let mut app = mock_app();
                let addr = setup_fee_contract(&mut app);

                let msg = ExecuteMsg::SetFee {
                    fee_type: Fees::Fixed,
                    module_name: Modules::Hub.to_string(),
                    fee_name: "creation".to_string(),
                    data: to_binary(&FixedPayment {
                        address: Some(COMMUNITY.to_string()),
                        value: Uint128::new(0),
                    })
                    .unwrap(),
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::InvalidFee {}.to_string()
                )
            }
        }
    }

    mod remove_fee {
        use super::*;

        #[test]
        fn test_happy_path_percentage() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);

            setup_fee(
                &mut app,
                addr.clone(),
                Fees::Percentage,
                Modules::Marketplace.as_str(),
                "komple",
                to_binary(&PercentagePayment {
                    value: Decimal::from_str("0.5").unwrap(),
                    address: Some(KOMPLE.to_string()),
                })
                .unwrap(),
            );

            let msg = ExecuteMsg::RemoveFee {
                fee_type: Fees::Percentage,
                module_name: Modules::Marketplace.to_string(),
                fee_name: "komple".to_string(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &[])
                .unwrap();

            let msg = QueryMsg::PercentageFee {
                module_name: Modules::Marketplace.to_string(),
                fee_name: "komple".to_string(),
            };
            let res: Result<PercentageFeeResponse, StdError> =
                app.wrap().query_wasm_smart(addr, &msg);
            assert!(res.is_err());
        }

        #[test]
        fn test_happy_path_fixed() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);

            setup_fee(
                &mut app,
                addr.clone(),
                Fees::Fixed,
                Modules::Hub.as_str(),
                "creation",
                to_binary(&FixedPayment {
                    value: Uint128::new(1_000_000),
                    address: Some(COMMUNITY.to_string()),
                })
                .unwrap(),
            );

            let msg = ExecuteMsg::RemoveFee {
                fee_type: Fees::Fixed,
                module_name: Modules::Hub.to_string(),
                fee_name: "creation".to_string(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &[])
                .unwrap();

            let msg = QueryMsg::PercentageFee {
                module_name: Modules::Hub.to_string(),
                fee_name: "creation".to_string(),
            };
            let res: Result<PercentageFeeResponse, StdError> =
                app.wrap().query_wasm_smart(addr, &msg);
            assert!(res.is_err());
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);

            setup_fee(
                &mut app,
                addr.clone(),
                Fees::Fixed,
                Modules::Hub.as_str(),
                "creation",
                to_binary(&FixedPayment {
                    value: Uint128::new(1_000_000),
                    address: Some(COMMUNITY.to_string()),
                })
                .unwrap(),
            );

            let msg = ExecuteMsg::RemoveFee {
                fee_type: Fees::Fixed,
                module_name: Modules::Hub.to_string(),
                fee_name: "creation".to_string(),
            };
            let err = app
                .execute_contract(Addr::unchecked(PAYMENT), addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }
    }

    mod distribute {
        use super::*;

        mod percentage {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let addr = setup_fee_contract(&mut app);

                setup_fee(
                    &mut app,
                    addr.clone(),
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
                    addr.clone(),
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
                    addr.clone(),
                    Fees::Percentage,
                    Modules::Marketplace.as_str(),
                    "payment",
                    to_binary(&PercentagePayment {
                        value: Decimal::from_str("0.03").unwrap(),
                        address: Some(PAYMENT.to_string()),
                    })
                    .unwrap(),
                );

                let msg = ExecuteMsg::Distribute {
                    fee_type: Fees::Percentage,
                    module_name: Modules::Marketplace.to_string(),
                    custom_payment_addresses: None,
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        addr.clone(),
                        &msg,
                        &[coin(360_000, NATIVE_DENOM)],
                    )
                    .unwrap();

                // 4_000_000 purchase price
                // 360_000 fee taken
                // 160_000 for komple
                // 120_000 for payment
                // 80_000 for community

                let balance = app.wrap().query_balance(KOMPLE, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(160_000));
                let balance = app.wrap().query_balance(COMMUNITY, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(80_000));
                let balance = app.wrap().query_balance(PAYMENT, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(120_000));

                let msg = ExecuteMsg::Distribute {
                    fee_type: Fees::Percentage,
                    module_name: Modules::Marketplace.to_string(),
                    custom_payment_addresses: None,
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        addr,
                        &msg,
                        &[coin(630_000, NATIVE_DENOM)],
                    )
                    .unwrap();

                // 7_000_000 purchase price
                // 630_000 fee taken
                // 280_000 for komple
                // 140_000 for community
                // 210_000 for payment

                let balance = app.wrap().query_balance(KOMPLE, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(440_000));
                let balance = app.wrap().query_balance(COMMUNITY, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(220_000));
                let balance = app.wrap().query_balance(PAYMENT, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(330_000));
            }

            #[test]
            fn test_custom_addresses() {
                let mut app = mock_app();
                let addr = setup_fee_contract(&mut app);

                setup_fee(
                    &mut app,
                    addr.clone(),
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
                    addr.clone(),
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
                    addr.clone(),
                    Fees::Percentage,
                    Modules::Marketplace.as_str(),
                    "payment",
                    to_binary(&PercentagePayment {
                        value: Decimal::from_str("0.03").unwrap(),
                        address: Some(PAYMENT.to_string()),
                    })
                    .unwrap(),
                );

                const NEW_COMMUNITY: &str = "new_community";
                const NEW_PAYMENT: &str = "new_payment";

                let msg = ExecuteMsg::Distribute {
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
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        addr,
                        &msg,
                        &[coin(360_000, NATIVE_DENOM)],
                    )
                    .unwrap();

                let balance = app.wrap().query_balance(COMMUNITY, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(0));
                let balance = app
                    .wrap()
                    .query_balance(NEW_COMMUNITY, NATIVE_DENOM)
                    .unwrap();
                assert_eq!(balance.amount, Uint128::new(80_000));
                let balance = app.wrap().query_balance(PAYMENT, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(0));
                let balance = app.wrap().query_balance(NEW_PAYMENT, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(120_000));
            }

            #[test]
            fn test_missing_payments() {
                let mut app = mock_app();
                let addr = setup_fee_contract(&mut app);

                setup_fee(
                    &mut app,
                    addr.clone(),
                    Fees::Percentage,
                    Modules::Marketplace.as_str(),
                    "komple",
                    to_binary(&PercentagePayment {
                        value: Decimal::from_str("0.04").unwrap(),
                        address: Some(KOMPLE.to_string()),
                    })
                    .unwrap(),
                );

                let msg = ExecuteMsg::Distribute {
                    fee_type: Fees::Percentage,
                    module_name: Modules::Hub.to_string(),
                    custom_payment_addresses: None,
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        addr,
                        &msg,
                        &[coin(360_000, NATIVE_DENOM)],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::NoPaymentsFound {}.to_string()
                );
            }
        }

        mod fixed {
            use komple_utils::funds::FundsError;

            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let addr = setup_fee_contract(&mut app);

                setup_fee(
                    &mut app,
                    addr.clone(),
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
                    addr.clone(),
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
                    addr.clone(),
                    Fees::Fixed,
                    Modules::Hub.as_str(),
                    "random",
                    to_binary(&FixedPayment {
                        value: Uint128::new(250_000),
                        address: Some(PAYMENT.to_string()),
                    })
                    .unwrap(),
                );

                let msg = ExecuteMsg::Distribute {
                    fee_type: Fees::Fixed,
                    module_name: Modules::Hub.to_string(),
                    custom_payment_addresses: None,
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        addr,
                        &msg,
                        &[coin(1_750_000, NATIVE_DENOM)],
                    )
                    .unwrap();

                let balance = app.wrap().query_balance(COMMUNITY, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(1_500_000));
                let balance = app.wrap().query_balance(PAYMENT, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(250_000));
            }

            #[test]
            fn test_custom_addresses() {
                let mut app = mock_app();
                let addr = setup_fee_contract(&mut app);

                setup_fee(
                    &mut app,
                    addr.clone(),
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
                    addr.clone(),
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
                    addr.clone(),
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

                let msg = ExecuteMsg::Distribute {
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
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        addr,
                        &msg,
                        &[coin(1_750_000, NATIVE_DENOM)],
                    )
                    .unwrap();

                let balance = app
                    .wrap()
                    .query_balance(NEW_COMMUNITY, NATIVE_DENOM)
                    .unwrap();
                assert_eq!(balance.amount, Uint128::new(1_500_000));
                let balance = app.wrap().query_balance(NEW_PAYMENT, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(250_000));
                let balance = app.wrap().query_balance(COMMUNITY, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(0));
                let balance = app.wrap().query_balance(PAYMENT, NATIVE_DENOM).unwrap();
                assert_eq!(balance.amount, Uint128::new(0));
            }

            #[test]
            fn test_missing_payments() {
                let mut app = mock_app();
                let addr = setup_fee_contract(&mut app);

                setup_fee(
                    &mut app,
                    addr.clone(),
                    Fees::Fixed,
                    Modules::Hub.as_str(),
                    "creation",
                    to_binary(&FixedPayment {
                        value: Uint128::new(1_000_000),
                        address: Some(COMMUNITY.to_string()),
                    })
                    .unwrap(),
                );

                let msg = ExecuteMsg::Distribute {
                    fee_type: Fees::Fixed,
                    module_name: Modules::Mint.to_string(),
                    custom_payment_addresses: None,
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        addr,
                        &msg,
                        &[coin(1_750_000, NATIVE_DENOM)],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::NoPaymentsFound {}.to_string()
                );
            }

            #[test]
            fn test_invalid_funds() {
                let mut app = mock_app();
                let addr = setup_fee_contract(&mut app);

                setup_fee(
                    &mut app,
                    addr.clone(),
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
                    addr.clone(),
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
                    addr.clone(),
                    Fees::Fixed,
                    Modules::Hub.as_str(),
                    "random",
                    to_binary(&FixedPayment {
                        value: Uint128::new(250_000),
                        address: Some(PAYMENT.to_string()),
                    })
                    .unwrap(),
                );

                let msg = ExecuteMsg::Distribute {
                    fee_type: Fees::Fixed,
                    module_name: Modules::Hub.to_string(),
                    custom_payment_addresses: None,
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    FundsError::MissingFunds {}.to_string()
                );

                let msg = ExecuteMsg::Distribute {
                    fee_type: Fees::Fixed,
                    module_name: Modules::Hub.to_string(),
                    custom_payment_addresses: None,
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        addr,
                        &msg,
                        &[coin(1_000_000, NATIVE_DENOM)],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    FundsError::InvalidFunds {
                        got: "1000000".to_string(),
                        expected: "1750000".to_string()
                    }
                    .to_string()
                );
            }
        }
    }

    mod lock_execute {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);

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

mod queries {
    use super::*;

    mod total_fees {
        use super::*;

        #[test]
        fn test_total_percentage_fees() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);

            setup_fee(
                &mut app,
                addr.clone(),
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
                addr.clone(),
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
                addr.clone(),
                Fees::Percentage,
                Modules::Marketplace.as_str(),
                "payment",
                to_binary(&PercentagePayment {
                    value: Decimal::from_str("0.03").unwrap(),
                    address: Some(PAYMENT.to_string()),
                })
                .unwrap(),
            );

            let msg = QueryMsg::TotalPercentageFees {
                module_name: Modules::Marketplace.to_string(),
                start_after: None,
                limit: None,
            };
            let res: ResponseWrapper<Decimal> = app.wrap().query_wasm_smart(addr, &msg).unwrap();
            assert_eq!(res.data, Decimal::from_str("0.09").unwrap());
        }

        #[test]
        fn test_total_fixed_fees() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);

            setup_fee(
                &mut app,
                addr.clone(),
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
                addr.clone(),
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
                addr.clone(),
                Fees::Fixed,
                Modules::Hub.as_str(),
                "random",
                to_binary(&FixedPayment {
                    value: Uint128::new(250_000),
                    address: Some(PAYMENT.to_string()),
                })
                .unwrap(),
            );

            let msg = QueryMsg::TotalFixedFees {
                module_name: Modules::Hub.to_string(),
                start_after: None,
                limit: None,
            };
            let res: ResponseWrapper<Uint128> = app.wrap().query_wasm_smart(addr, &msg).unwrap();
            assert_eq!(res.data, Uint128::new(1_750_000));
        }
    }

    mod percentage_fees {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);

            setup_fee(
                &mut app,
                addr.clone(),
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
                addr.clone(),
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
                addr.clone(),
                Fees::Percentage,
                Modules::Marketplace.as_str(),
                "payment",
                to_binary(&PercentagePayment {
                    value: Decimal::from_str("0.03").unwrap(),
                    address: Some(PAYMENT.to_string()),
                })
                .unwrap(),
            );
            setup_fee(
                &mut app,
                addr.clone(),
                Fees::Percentage,
                Modules::Marketplace.as_str(),
                "test",
                to_binary(&PercentagePayment {
                    value: Decimal::from_str("0.1").unwrap(),
                    address: Some("test".to_string()),
                })
                .unwrap(),
            );

            let msg = QueryMsg::PercentageFees {
                module_name: Modules::Marketplace.to_string(),
                start_after: None,
                limit: None,
            };
            let res: ResponseWrapper<Vec<PercentageFeeResponse>> =
                app.wrap().query_wasm_smart(addr.clone(), &msg).unwrap();
            assert_eq!(res.data.len(), 4);
            assert_eq!(res.data[0].fee_name, "community");
            assert_eq!(res.data[0].address, Some(COMMUNITY.to_string()));
            assert_eq!(res.data[0].value, Decimal::from_str("0.02").unwrap());
            assert_eq!(res.data[2].fee_name, "payment");
            assert_eq!(res.data[2].address, Some(PAYMENT.to_string()));
            assert_eq!(res.data[2].value, Decimal::from_str("0.03").unwrap());

            let msg = QueryMsg::PercentageFees {
                module_name: Modules::Marketplace.to_string(),
                start_after: Some("community".to_string()),
                limit: None,
            };
            let res: ResponseWrapper<Vec<PercentageFeeResponse>> =
                app.wrap().query_wasm_smart(addr.clone(), &msg).unwrap();
            assert_eq!(res.data.len(), 3);
            assert_eq!(res.data[0].fee_name, "komple");
            assert_eq!(res.data[0].address, Some(KOMPLE.to_string()));
            assert_eq!(res.data[0].value, Decimal::from_str("0.04").unwrap());
            assert_eq!(res.data[1].fee_name, "payment");
            assert_eq!(res.data[1].address, Some(PAYMENT.to_string()));
            assert_eq!(res.data[1].value, Decimal::from_str("0.03").unwrap());

            let msg = QueryMsg::PercentageFees {
                module_name: Modules::Marketplace.to_string(),
                start_after: Some("komple".to_string()),
                limit: Some(1),
            };
            let res: ResponseWrapper<Vec<PercentageFeeResponse>> =
                app.wrap().query_wasm_smart(addr, &msg).unwrap();
            assert_eq!(res.data.len(), 1);
            assert_eq!(res.data[0].fee_name, "payment");
            assert_eq!(res.data[0].address, Some(PAYMENT.to_string()));
            assert_eq!(res.data[0].value, Decimal::from_str("0.03").unwrap());
        }

        #[test]
        fn test_filters() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);

            setup_fee(
                &mut app,
                addr.clone(),
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
                addr.clone(),
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
                addr.clone(),
                Fees::Percentage,
                Modules::Marketplace.as_str(),
                "payment",
                to_binary(&PercentagePayment {
                    value: Decimal::from_str("0.03").unwrap(),
                    address: Some(PAYMENT.to_string()),
                })
                .unwrap(),
            );

            let msg = QueryMsg::PercentageFees {
                module_name: Modules::Marketplace.to_string(),
                start_after: Some("community".to_string()),
                limit: None,
            };
            let res: ResponseWrapper<Vec<PercentageFeeResponse>> =
                app.wrap().query_wasm_smart(addr.clone(), &msg).unwrap();
            assert_eq!(res.data.len(), 2);
            assert_eq!(res.data[0].fee_name, "komple");
            assert_eq!(res.data[0].address, Some(KOMPLE.to_string()));
            assert_eq!(res.data[0].value, Decimal::from_str("0.04").unwrap());
            assert_eq!(res.data[1].fee_name, "payment");
            assert_eq!(res.data[1].address, Some(PAYMENT.to_string()));
            assert_eq!(res.data[1].value, Decimal::from_str("0.03").unwrap());

            let msg = QueryMsg::PercentageFees {
                module_name: Modules::Marketplace.to_string(),
                start_after: Some("community".to_string()),
                limit: Some(1),
            };
            let res: ResponseWrapper<Vec<PercentageFeeResponse>> =
                app.wrap().query_wasm_smart(addr, &msg).unwrap();
            assert_eq!(res.data.len(), 1);
            assert_eq!(res.data[0].fee_name, "komple");
            assert_eq!(res.data[0].address, Some(KOMPLE.to_string()));
            assert_eq!(res.data[0].value, Decimal::from_str("0.04").unwrap());
        }
    }

    mod fixed_fees {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);

            setup_fee(
                &mut app,
                addr.clone(),
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
                addr.clone(),
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
                addr.clone(),
                Fees::Fixed,
                Modules::Hub.as_str(),
                "random",
                to_binary(&FixedPayment {
                    value: Uint128::new(250_000),
                    address: Some(PAYMENT.to_string()),
                })
                .unwrap(),
            );
            setup_fee(
                &mut app,
                addr.clone(),
                Fees::Fixed,
                Modules::Hub.as_str(),
                "test",
                to_binary(&FixedPayment {
                    value: Uint128::new(100_000),
                    address: Some("test".to_string()),
                })
                .unwrap(),
            );

            let msg = QueryMsg::FixedFees {
                module_name: Modules::Hub.to_string(),
                start_after: None,
                limit: None,
            };
            let res: ResponseWrapper<Vec<FixedFeeResponse>> =
                app.wrap().query_wasm_smart(addr.clone(), &msg).unwrap();
            assert_eq!(res.data.len(), 4);
            assert_eq!(res.data[0].fee_name, "creation");
            assert_eq!(res.data[0].address, Some(COMMUNITY.to_string()));
            assert_eq!(res.data[0].value, Uint128::new(1_000_000));
            assert_eq!(res.data[2].fee_name, "random");
            assert_eq!(res.data[2].address, Some(PAYMENT.to_string()));
            assert_eq!(res.data[2].value, Uint128::new(250_000));

            let msg = QueryMsg::FixedFees {
                module_name: Modules::Hub.to_string(),
                start_after: Some("creation".to_string()),
                limit: None,
            };
            let res: ResponseWrapper<Vec<FixedFeeResponse>> =
                app.wrap().query_wasm_smart(addr.clone(), &msg).unwrap();
            assert_eq!(res.data.len(), 3);
            assert_eq!(res.data[0].fee_name, "module_register");
            assert_eq!(res.data[0].address, Some(COMMUNITY.to_string()));
            assert_eq!(res.data[0].value, Uint128::new(500_000));
            assert_eq!(res.data[2].fee_name, "test");
            assert_eq!(res.data[2].address, Some("test".to_string()));
            assert_eq!(res.data[2].value, Uint128::new(100_000));

            let msg = QueryMsg::FixedFees {
                module_name: Modules::Hub.to_string(),
                start_after: Some("module_register".to_string()),
                limit: Some(1),
            };
            let res: ResponseWrapper<Vec<FixedFeeResponse>> =
                app.wrap().query_wasm_smart(addr, &msg).unwrap();
            assert_eq!(res.data.len(), 1);
            assert_eq!(res.data[0].fee_name, "random");
            assert_eq!(res.data[0].address, Some(PAYMENT.to_string()));
            assert_eq!(res.data[0].value, Uint128::new(250_000));
        }

        #[test]
        fn test_filters() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);

            setup_fee(
                &mut app,
                addr.clone(),
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
                addr.clone(),
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
                addr.clone(),
                Fees::Fixed,
                Modules::Hub.as_str(),
                "random",
                to_binary(&FixedPayment {
                    value: Uint128::new(250_000),
                    address: Some(PAYMENT.to_string()),
                })
                .unwrap(),
            );

            let msg = QueryMsg::FixedFees {
                module_name: Modules::Hub.to_string(),
                start_after: Some("creation".to_string()),
                limit: None,
            };
            let res: ResponseWrapper<Vec<FixedFeeResponse>> =
                app.wrap().query_wasm_smart(addr.clone(), &msg).unwrap();
            assert_eq!(res.data.len(), 2);
            assert_eq!(res.data[0].fee_name, "module_register");
            assert_eq!(res.data[0].address, Some(COMMUNITY.to_string()));
            assert_eq!(res.data[0].value, Uint128::new(500_000));
            assert_eq!(res.data[1].fee_name, "random");
            assert_eq!(res.data[1].address, Some(PAYMENT.to_string()));
            assert_eq!(res.data[1].value, Uint128::new(250_000));

            let msg = QueryMsg::FixedFees {
                module_name: Modules::Hub.to_string(),
                start_after: Some("creation".to_string()),
                limit: Some(1),
            };
            let res: ResponseWrapper<Vec<FixedFeeResponse>> =
                app.wrap().query_wasm_smart(addr, &msg).unwrap();
            assert_eq!(res.data.len(), 1);
            assert_eq!(res.data[0].fee_name, "module_register");
            assert_eq!(res.data[0].address, Some(COMMUNITY.to_string()));
            assert_eq!(res.data[0].value, Uint128::new(500_000));
        }
    }

    mod all_keys {
        use super::*;

        mod percentage {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let addr = setup_fee_contract(&mut app);

                let msg = QueryMsg::TotalPercentageFees {
                    module_name: Modules::Marketplace.to_string(),
                    start_after: None,
                    limit: None,
                };
                let res: ResponseWrapper<Decimal> =
                    app.wrap().query_wasm_smart(addr.clone(), &msg).unwrap();
                assert!(res.data.is_zero());

                setup_fee(
                    &mut app,
                    addr.clone(),
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
                    addr.clone(),
                    Fees::Percentage,
                    Modules::Hub.as_str(),
                    "komple",
                    to_binary(&PercentagePayment {
                        value: Decimal::from_str("0.04").unwrap(),
                        address: Some(KOMPLE.to_string()),
                    })
                    .unwrap(),
                );
                setup_fee(
                    &mut app,
                    addr.clone(),
                    Fees::Percentage,
                    Modules::Mint.as_str(),
                    "community",
                    to_binary(&PercentagePayment {
                        value: Decimal::from_str("0.02").unwrap(),
                        address: Some(COMMUNITY.to_string()),
                    })
                    .unwrap(),
                );

                let msg = QueryMsg::Keys {
                    fee_type: Fees::Percentage,
                    start_after: None,
                    limit: None,
                };
                let res: ResponseWrapper<Vec<String>> =
                    app.wrap().query_wasm_smart(addr, &msg).unwrap();
                assert_eq!(res.data.len(), 3);
                assert_eq!(res.data[0], "marketplace");
                assert_eq!(res.data[1], "mint");
                assert_eq!(res.data[2], "hub");
            }

            #[test]
            fn test_filters() {
                let mut app = mock_app();
                let addr = setup_fee_contract(&mut app);

                setup_fee(
                    &mut app,
                    addr.clone(),
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
                    addr.clone(),
                    Fees::Percentage,
                    Modules::Hub.as_str(),
                    "komple",
                    to_binary(&PercentagePayment {
                        value: Decimal::from_str("0.04").unwrap(),
                        address: Some(KOMPLE.to_string()),
                    })
                    .unwrap(),
                );
                setup_fee(
                    &mut app,
                    addr.clone(),
                    Fees::Percentage,
                    Modules::Mint.as_str(),
                    "community",
                    to_binary(&PercentagePayment {
                        value: Decimal::from_str("0.02").unwrap(),
                        address: Some(COMMUNITY.to_string()),
                    })
                    .unwrap(),
                );

                let msg = QueryMsg::Keys {
                    fee_type: Fees::Percentage,
                    start_after: Some("marketplace".to_string()),
                    limit: Some(2),
                };
                let res: ResponseWrapper<Vec<String>> =
                    app.wrap().query_wasm_smart(addr, &msg).unwrap();
                assert_eq!(res.data.len(), 2);
                assert_eq!(res.data[0], "marketplace");
                assert_eq!(res.data[1], "mint");
            }
        }

        mod fixed {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let addr = setup_fee_contract(&mut app);

                let msg = QueryMsg::TotalFixedFees {
                    module_name: Modules::Marketplace.to_string(),
                    start_after: None,
                    limit: None,
                };
                let res: ResponseWrapper<Uint128> =
                    app.wrap().query_wasm_smart(addr.clone(), &msg).unwrap();
                assert!(res.data.is_zero());

                setup_fee(
                    &mut app,
                    addr.clone(),
                    Fees::Fixed,
                    Modules::Marketplace.as_str(),
                    "creation",
                    to_binary(&FixedPayment {
                        value: Uint128::new(1_000_000),
                        address: Some(COMMUNITY.to_string()),
                    })
                    .unwrap(),
                );
                setup_fee(
                    &mut app,
                    addr.clone(),
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
                    addr.clone(),
                    Fees::Fixed,
                    Modules::Mint.as_str(),
                    "creation",
                    to_binary(&FixedPayment {
                        value: Uint128::new(1_000_000),
                        address: Some(COMMUNITY.to_string()),
                    })
                    .unwrap(),
                );

                let msg = QueryMsg::Keys {
                    fee_type: Fees::Fixed,
                    start_after: None,
                    limit: None,
                };
                let res: ResponseWrapper<Vec<String>> =
                    app.wrap().query_wasm_smart(addr, &msg).unwrap();
                assert_eq!(res.data.len(), 3);
                assert_eq!(res.data[0], "marketplace");
                assert_eq!(res.data[1], "mint");
                assert_eq!(res.data[2], "hub");
            }

            #[test]
            fn test_filters() {
                let mut app = mock_app();
                let addr = setup_fee_contract(&mut app);

                setup_fee(
                    &mut app,
                    addr.clone(),
                    Fees::Fixed,
                    Modules::Marketplace.as_str(),
                    "creation",
                    to_binary(&FixedPayment {
                        value: Uint128::new(1_000_000),
                        address: Some(COMMUNITY.to_string()),
                    })
                    .unwrap(),
                );
                setup_fee(
                    &mut app,
                    addr.clone(),
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
                    addr.clone(),
                    Fees::Fixed,
                    Modules::Mint.as_str(),
                    "creation",
                    to_binary(&FixedPayment {
                        value: Uint128::new(1_000_000),
                        address: Some(COMMUNITY.to_string()),
                    })
                    .unwrap(),
                );

                let msg = QueryMsg::Keys {
                    fee_type: Fees::Fixed,
                    start_after: Some("marketplace".to_string()),
                    limit: Some(2),
                };
                let res: ResponseWrapper<Vec<String>> =
                    app.wrap().query_wasm_smart(addr, &msg).unwrap();
                assert_eq!(res.data.len(), 2);
                assert_eq!(res.data[0], "marketplace");
                assert_eq!(res.data[1], "mint");
            }
        }
    }
}
