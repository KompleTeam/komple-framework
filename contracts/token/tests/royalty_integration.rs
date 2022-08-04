use cosmwasm_std::{Addr, Coin, Decimal, Empty, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_types::royalty::Royalty;
use komple_types::{collection::Collections, query::ResponseWrapper};
use royalty_contract::msg::{ExecuteMsg as RoyaltyExecuteMsg, QueryMsg as RoyaltyQueryMsg};
use std::str::FromStr;
use token_contract::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TokenInfo};
use token_contract::state::{CollectionInfo, Contracts};

pub fn token_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        token_contract::contract::execute,
        token_contract::contract::instantiate,
        token_contract::contract::query,
    )
    .with_reply(token_contract::contract::reply);
    Box::new(contract)
}

pub fn royalty_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        royalty_contract::contract::execute,
        royalty_contract::contract::instantiate,
        royalty_contract::contract::query,
    );
    Box::new(contract)
}

const USER: &str = "juno..user";
const ADMIN: &str = "juno..admin";
const RANDOM: &str = "juno..random";
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

fn token_contract_instantiation(app: &mut App) -> Addr {
    let token_code_id = app.store_code(token_contract());

    let collection_info = CollectionInfo {
        collection_type: Collections::Normal,
        name: "Test Collection".to_string(),
        description: "Test Collection".to_string(),
        image: "https://image.com".to_string(),
        external_link: None,
    };
    let token_info = TokenInfo {
        symbol: "TEST".to_string(),
        minter: ADMIN.to_string(),
    };
    let msg = InstantiateMsg {
        admin: ADMIN.to_string(),
        token_info,
        per_address_limit: None,
        start_time: None,
        collection_info,
        max_token_limit: None,
        unit_price: None,
        native_denom: NATIVE_DENOM.to_string(),
    };
    let token_contract_addr = app
        .instantiate_contract(
            token_code_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "test",
            None,
        )
        .unwrap();

    token_contract_addr
}

mod initialization {
    use super::*;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let token_contract_addr = token_contract_instantiation(&mut app);
        let royalty_code_id = app.store_code(royalty_contract());

        let msg = ExecuteMsg::InitRoyaltyContract {
            code_id: royalty_code_id,
            share: Decimal::from_str("0.5").unwrap(),
            royalty_type: Royalty::Admin,
        };
        let _ = app
            .execute_contract(
                Addr::unchecked(ADMIN),
                token_contract_addr.clone(),
                &msg,
                &vec![],
            )
            .unwrap();

        let msg = QueryMsg::Contracts {};
        let res: ResponseWrapper<Contracts> = app
            .wrap()
            .query_wasm_smart(token_contract_addr.clone(), &msg)
            .unwrap();
        assert_eq!(res.data.royalty.unwrap(), "contract1")
    }
}

mod actions {
    use super::*;

    mod update_token_royalty_address {
        use royalty_contract::msg::RoyaltyResponse;
        use token_contract::ContractError;

        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let token_contract_addr = token_contract_instantiation(&mut app);
            let royalty_code_id = app.store_code(royalty_contract());

            let msg = ExecuteMsg::Mint {
                owner: USER.to_string(),
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    token_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let msg = ExecuteMsg::InitRoyaltyContract {
                code_id: royalty_code_id,
                share: Decimal::from_str("0.5").unwrap(),
                royalty_type: Royalty::Tokens,
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    token_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let msg = QueryMsg::Contracts {};
            let res: ResponseWrapper<Contracts> = app
                .wrap()
                .query_wasm_smart(token_contract_addr.clone(), &msg)
                .unwrap();
            let royalty_contract_addr = res.data.royalty.unwrap();

            let msg = RoyaltyQueryMsg::Royalty {
                owner: USER.to_string(),
                collection_id: 1,
                token_id: 1,
            };
            let res: ResponseWrapper<RoyaltyResponse> = app
                .wrap()
                .query_wasm_smart(royalty_contract_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.address, USER);

            let msg = RoyaltyExecuteMsg::UpdateTokenRoyaltyAddress {
                collection_id: 1,
                token_id: 1,
                address: RANDOM.to_string(),
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(USER),
                    royalty_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let msg = RoyaltyQueryMsg::Royalty {
                owner: USER.to_string(),
                collection_id: 1,
                token_id: 1,
            };
            let res: ResponseWrapper<RoyaltyResponse> = app
                .wrap()
                .query_wasm_smart(royalty_contract_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.address, RANDOM);
        }

        #[test]
        fn test_invalid_owner() {
            let mut app = mock_app();
            let token_contract_addr = token_contract_instantiation(&mut app);
            let royalty_code_id = app.store_code(royalty_contract());

            let msg = ExecuteMsg::Mint {
                owner: USER.to_string(),
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    token_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let msg = ExecuteMsg::InitRoyaltyContract {
                code_id: royalty_code_id,
                share: Decimal::from_str("0.5").unwrap(),
                royalty_type: Royalty::Tokens,
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    token_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let msg = QueryMsg::Contracts {};
            let res: ResponseWrapper<Contracts> = app
                .wrap()
                .query_wasm_smart(token_contract_addr.clone(), &msg)
                .unwrap();
            let royalty_contract_addr = res.data.royalty.unwrap();

            let msg = RoyaltyExecuteMsg::UpdateTokenRoyaltyAddress {
                collection_id: 1,
                token_id: 1,
                address: RANDOM.to_string(),
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(RANDOM),
                    royalty_contract_addr.clone(),
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
}
