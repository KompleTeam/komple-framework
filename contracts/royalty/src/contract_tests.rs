#[cfg(test)]
mod tests {
    use crate::msg::{ExecuteMsg, QueryMsg};
    use crate::ContractError;
    use cosmwasm_std::{Addr, Coin, Decimal, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
    use rift_types::query::ResponseWrapper;
    use rift_types::royalty::Royalty;
    use std::str::FromStr;

    use crate::msg::InstantiateMsg;

    pub fn royalty_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    const USER: &str = "juno..user";
    const ADMIN: &str = "juno..admin";
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

    fn proper_instantiate(app: &mut App, share: Decimal, royalty_type: Royalty) -> Addr {
        let royalty_code_id = app.store_code(royalty_contract());

        let msg = InstantiateMsg {
            admin: ADMIN.to_string(),
            share,
            royalty_type,
        };
        let royalty_contract_addr = app
            .instantiate_contract(
                royalty_code_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        royalty_contract_addr
    }

    mod initialization {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let royalty_code_id = app.store_code(royalty_contract());

            let msg = InstantiateMsg {
                admin: ADMIN.to_string(),
                share: Decimal::from_str("0.5").unwrap(),
                royalty_type: Royalty::Admin,
            };
            let _ = app
                .instantiate_contract(
                    royalty_code_id,
                    Addr::unchecked(ADMIN),
                    &msg,
                    &[],
                    "test",
                    None,
                )
                .unwrap();
        }

        #[test]
        fn test_invalid_share() {
            let mut app = mock_app();
            let royalty_code_id = app.store_code(royalty_contract());

            let msg = InstantiateMsg {
                admin: ADMIN.to_string(),
                share: Decimal::from_str("1.1").unwrap(),
                royalty_type: Royalty::Admin,
            };
            let err = app
                .instantiate_contract(
                    royalty_code_id,
                    Addr::unchecked(ADMIN),
                    &msg,
                    &[],
                    "test",
                    None,
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::InvalidShare {}.to_string()
            )
        }
    }

    mod actions {
        use super::*;

        mod update_admin_royalty_address {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let royalty_contract_addr =
                    proper_instantiate(&mut app, Decimal::from_str("0.5").unwrap(), Royalty::Admin);

                let query_msg = QueryMsg::RoyaltyAddress {
                    owner: USER.to_string(),
                    collection_id: 1,
                    token_id: 1,
                };
                let res: ResponseWrapper<Addr> = app
                    .wrap()
                    .query_wasm_smart(royalty_contract_addr.clone(), &query_msg)
                    .unwrap();
                assert_eq!(res.data, Addr::unchecked(ADMIN));

                let update_msg = ExecuteMsg::UpdateOwnerRoyaltyAddress {
                    address: USER.to_string(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        royalty_contract_addr.clone(),
                        &update_msg,
                        &vec![],
                    )
                    .unwrap();

                let query_msg = QueryMsg::RoyaltyAddress {
                    owner: USER.to_string(),
                    collection_id: 1,
                    token_id: 1,
                };
                let res: ResponseWrapper<Addr> = app
                    .wrap()
                    .query_wasm_smart(royalty_contract_addr.clone(), &query_msg)
                    .unwrap();
                assert_eq!(res.data, Addr::unchecked(ADMIN));
            }
        }

        mod update_owner_royalty_address {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let royalty_contract_addr = proper_instantiate(
                    &mut app,
                    Decimal::from_str("0.5").unwrap(),
                    Royalty::Owners,
                );

                let msg = ExecuteMsg::UpdateOwnerRoyaltyAddress {
                    address: USER.to_string(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(USER),
                        royalty_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = QueryMsg::RoyaltyAddress {
                    owner: USER.to_string(),
                    collection_id: 1,
                    token_id: 1,
                };
                let res: ResponseWrapper<Addr> = app
                    .wrap()
                    .query_wasm_smart(royalty_contract_addr.clone(), &msg)
                    .unwrap();
                assert_eq!(res.data, Addr::unchecked(USER));
            }
        }
    }
}
