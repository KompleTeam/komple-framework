use crate::msg::{CustomAddress, ShareResponse};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::Config;
use crate::ContractError;
use cosmwasm_std::Decimal;
use cosmwasm_std::{coin, Addr, Empty, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
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
                vec![coin(1_000_000, NATIVE_DENOM)],
            )
            .unwrap();
    })
}

fn setup_fee_contract(app: &mut App) -> Addr {
    let code_id = app.store_code(fee_contract());

    let msg = InstantiateMsg {};
    let addr = app
        .instantiate_contract(code_id, Addr::unchecked(ADMIN), &msg, &vec![], "test", None)
        .unwrap();

    addr
}

fn setup_share(
    app: &mut App,
    contract: Addr,
    name: &str,
    address: Option<String>,
    percentage: &str,
) {
    let msg = ExecuteMsg::AddShare {
        name: name.to_string(),
        address,
        percentage: Decimal::from_str(percentage).unwrap(),
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), contract, &msg, &vec![])
        .unwrap();
}

mod instantiation {
    use super::*;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let code_id = app.store_code(fee_contract());

        let msg = InstantiateMsg {};
        let addr = app
            .instantiate_contract(code_id, Addr::unchecked(ADMIN), &msg, &vec![], "test", None)
            .unwrap();

        let msg = QueryMsg::Config {};
        let res: Config = app.wrap().query_wasm_smart(addr, &msg).unwrap();
        assert_eq!(res.admin, ADMIN.to_string());
    }
}

mod actions {
    use super::*;

    mod add_share {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);

            let msg = ExecuteMsg::AddShare {
                name: "komple".to_string(),
                address: Some(KOMPLE.to_string()),
                percentage: Decimal::from_str("0.04").unwrap(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &vec![])
                .unwrap();

            let msg = QueryMsg::Share {
                name: "komple".to_string(),
            };
            let res: ShareResponse = app.wrap().query_wasm_smart(addr.clone(), &msg).unwrap();
            assert_eq!(res.name, "komple");
            assert_eq!(res.payment_address, Some(KOMPLE.to_string()));
            assert_eq!(res.fee_percentage, Decimal::from_str("0.04").unwrap());
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);

            let msg = ExecuteMsg::AddShare {
                name: "komple".to_string(),
                address: Some(KOMPLE.to_string()),
                percentage: Decimal::from_str("0.04").unwrap(),
            };
            let err = app
                .execute_contract(Addr::unchecked(PAYMENT), addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }

        #[test]
        fn test_existing_share() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);

            let msg = ExecuteMsg::AddShare {
                name: "komple".to_string(),
                address: Some(KOMPLE.to_string()),
                percentage: Decimal::from_str("0.04").unwrap(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &vec![])
                .unwrap();

            let msg = ExecuteMsg::AddShare {
                name: "komple".to_string(),
                address: Some(KOMPLE.to_string()),
                percentage: Decimal::from_str("0.04").unwrap(),
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::ExistingShare {}.to_string()
            )
        }

        #[test]
        fn test_invalid_fee_percentage() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);

            let msg = ExecuteMsg::AddShare {
                name: "komple".to_string(),
                address: Some(KOMPLE.to_string()),
                percentage: Decimal::from_str("1.1").unwrap(),
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::InvalidPercentage {}.to_string()
            )
        }

        #[test]
        fn test_invalid_total_fee() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);

            setup_share(
                &mut app,
                addr.clone(),
                "share_1",
                Some("share_1".to_string()),
                "0.4",
            );
            setup_share(
                &mut app,
                addr.clone(),
                "share_2",
                Some("share_2".to_string()),
                "0.4",
            );

            let msg = ExecuteMsg::AddShare {
                name: "komple".to_string(),
                address: Some(KOMPLE.to_string()),
                percentage: Decimal::from_str("0.2").unwrap(),
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::InvalidTotalFee {}.to_string()
            );

            let msg = ExecuteMsg::AddShare {
                name: "komple".to_string(),
                address: Some(KOMPLE.to_string()),
                percentage: Decimal::from_str("0.3").unwrap(),
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::InvalidTotalFee {}.to_string()
            )
        }
    }

    mod update_share {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);
            setup_share(
                &mut app,
                addr.clone(),
                "komple",
                Some(KOMPLE.to_string()),
                "0.1",
            );

            let msg = ExecuteMsg::UpdateShare {
                name: "komple".to_string(),
                address: Some(PAYMENT.to_string()),
                percentage: Decimal::from_str("0.28").unwrap(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &vec![])
                .unwrap();

            let msg = QueryMsg::Share {
                name: "komple".to_string(),
            };
            let res: ShareResponse = app.wrap().query_wasm_smart(addr.clone(), &msg).unwrap();
            assert_eq!(res.payment_address, Some(PAYMENT.to_string()));
            assert_eq!(res.fee_percentage, Decimal::from_str("0.28").unwrap());
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);
            setup_share(
                &mut app,
                addr.clone(),
                "komple",
                Some(KOMPLE.to_string()),
                "0.1",
            );

            let msg = ExecuteMsg::UpdateShare {
                name: "komple".to_string(),
                address: Some(PAYMENT.to_string()),
                percentage: Decimal::from_str("0.28").unwrap(),
            };
            let err = app
                .execute_contract(Addr::unchecked(PAYMENT), addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }

        #[test]
        fn test_missing_share() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);
            setup_share(
                &mut app,
                addr.clone(),
                "komple",
                Some(KOMPLE.to_string()),
                "0.1",
            );

            let msg = ExecuteMsg::UpdateShare {
                name: "missing".to_string(),
                address: Some(PAYMENT.to_string()),
                percentage: Decimal::from_str("0.28").unwrap(),
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::ShareNotFound {}.to_string()
            );
        }

        #[test]
        fn test_invalid_percentage() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);
            setup_share(
                &mut app,
                addr.clone(),
                "komple",
                Some(KOMPLE.to_string()),
                "0.1",
            );

            let msg = ExecuteMsg::UpdateShare {
                name: "komple".to_string(),
                address: Some(PAYMENT.to_string()),
                percentage: Decimal::from_str("1.2").unwrap(),
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::InvalidPercentage {}.to_string()
            );
        }

        #[test]
        fn test_invalid_total_fee() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);

            setup_share(
                &mut app,
                addr.clone(),
                "share_1",
                Some("share_1".to_string()),
                "0.4",
            );
            setup_share(
                &mut app,
                addr.clone(),
                "share_2",
                Some("share_2".to_string()),
                "0.4",
            );

            let msg = ExecuteMsg::UpdateShare {
                name: "share_1".to_string(),
                address: Some("share_1".to_string()),
                percentage: Decimal::from_str("0.6").unwrap(),
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::InvalidTotalFee {}.to_string()
            );

            let msg = ExecuteMsg::UpdateShare {
                name: "share_1".to_string(),
                address: Some("share_1".to_string()),
                percentage: Decimal::from_str("0.7").unwrap(),
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::InvalidTotalFee {}.to_string()
            );
        }
    }

    mod distribute {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let addr = setup_fee_contract(&mut app);

            setup_share(
                &mut app,
                addr.clone(),
                "komple",
                Some(KOMPLE.to_string()),
                "0.04",
            );
            setup_share(
                &mut app,
                addr.clone(),
                "community",
                Some(COMMUNITY.to_string()),
                "0.02",
            );
            setup_share(
                &mut app,
                addr.clone(),
                "test",
                Some(PAYMENT.to_string()),
                "0.03",
            );

            let msg = ExecuteMsg::Distribute {
                custom_addresses: None,
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    addr.clone(),
                    &msg,
                    &vec![coin(360_000, NATIVE_DENOM)],
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
                custom_addresses: None,
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    addr.clone(),
                    &msg,
                    &vec![coin(630_000, NATIVE_DENOM)],
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

            setup_share(
                &mut app,
                addr.clone(),
                "komple",
                Some(KOMPLE.to_string()),
                "0.04",
            );
            setup_share(
                &mut app,
                addr.clone(),
                "community",
                Some(COMMUNITY.to_string()),
                "0.02",
            );
            setup_share(
                &mut app,
                addr.clone(),
                "test",
                Some(PAYMENT.to_string()),
                "0.03",
            );

            const NEW_COMMUNITY: &str = "new_community";
            const NEW_PAYMENT: &str = "new_payment";

            let msg = ExecuteMsg::Distribute {
                custom_addresses: Some(vec![
                    CustomAddress {
                        name: "community".to_string(),
                        payment_address: NEW_COMMUNITY.to_string(),
                    },
                    CustomAddress {
                        name: "test".to_string(),
                        payment_address: NEW_PAYMENT.to_string(),
                    },
                ]),
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    addr.clone(),
                    &msg,
                    &vec![coin(360_000, NATIVE_DENOM)],
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
    }
}

mod queries {
    use super::*;

    #[test]
    fn test_multiple_shares() {
        let mut app = mock_app();
        let addr = setup_fee_contract(&mut app);

        setup_share(
            &mut app,
            addr.clone(),
            "komple",
            Some(KOMPLE.to_string()),
            "0.3",
        );
        setup_share(
            &mut app,
            addr.clone(),
            "community_pool",
            Some(COMMUNITY.to_string()),
            "0.2",
        );
        setup_share(
            &mut app,
            addr.clone(),
            "test",
            Some(PAYMENT.to_string()),
            "0.4",
        );

        let msg = QueryMsg::Shares {};
        let res: Vec<ShareResponse> = app.wrap().query_wasm_smart(addr.clone(), &msg).unwrap();

        assert_eq!(res.len(), 3);

        assert_eq!(res[1].name, "komple");
        assert_eq!(res[1].fee_percentage, Decimal::from_str("0.3").unwrap());
        assert_eq!(res[1].payment_address, Some(KOMPLE.to_string()));

        assert_eq!(res[0].name, "community_pool");
        assert_eq!(res[0].fee_percentage, Decimal::from_str("0.2").unwrap());
        assert_eq!(res[0].payment_address, Some(COMMUNITY.to_string()));

        assert_eq!(res[2].name, "test");
        assert_eq!(res[2].fee_percentage, Decimal::from_str("0.4").unwrap());
        assert_eq!(res[2].payment_address, Some(PAYMENT.to_string()));
    }

    #[test]
    fn test_total_fee() {
        let mut app = mock_app();
        let addr = setup_fee_contract(&mut app);

        setup_share(
            &mut app,
            addr.clone(),
            "komple",
            Some(KOMPLE.to_string()),
            "0.3",
        );
        setup_share(
            &mut app,
            addr.clone(),
            "community_pool",
            Some(COMMUNITY.to_string()),
            "0.2",
        );
        setup_share(
            &mut app,
            addr.clone(),
            "test",
            Some(PAYMENT.to_string()),
            "0.4",
        );

        let msg = QueryMsg::TotalFee {};
        let res: Decimal = app.wrap().query_wasm_smart(addr.clone(), &msg).unwrap();
        assert_eq!(res, Decimal::from_str("0.9").unwrap())
    }
}
