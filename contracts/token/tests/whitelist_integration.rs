use cosmwasm_std::{coin, Timestamp};
use cosmwasm_std::{Addr, Coin, Empty, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use rift_types::{collection::Collections, query::ResponseWrapper};
use rift_utils::query_token_owner;
use token_contract::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TokenInfo};
use token_contract::state::{CollectionInfo, Contracts};
use token_contract::ContractError;
use whitelist_contract::msg::InstantiateMsg as WhitelistInstantiateMsg;

pub fn token_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        token_contract::contract::execute,
        token_contract::contract::instantiate,
        token_contract::contract::query,
    )
    .with_reply(token_contract::contract::reply);
    Box::new(contract)
}

pub fn whitelist_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        whitelist_contract::contract::execute,
        whitelist_contract::contract::instantiate,
        whitelist_contract::contract::query,
    );
    Box::new(contract)
}

const USER: &str = "juno..user";
const ADMIN: &str = "juno..admin";
const RANDOM: &str = "juno..random";
const RANDOM_2: &str = "juno..random2";
const NATIVE_DENOM: &str = "denom";

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

fn setup_whitelist(
    app: &mut App,
    token_contract_addr: Addr,
    members: Vec<String>,
    start_time: Timestamp,
    end_time: Timestamp,
    unit_price: Coin,
    per_address_limit: u8,
) -> Addr {
    let whitelist_code_id = app.store_code(whitelist_contract());

    let instantiate_msg = WhitelistInstantiateMsg {
        start_time,
        end_time,
        members,
        unit_price,
        per_address_limit,
        member_limit: 10,
    };
    let msg = ExecuteMsg::InitWhitelistContract {
        code_id: whitelist_code_id,
        instantiate_msg,
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

    res.data.whitelist.unwrap()
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
        let whitelist_code_id = app.store_code(whitelist_contract());

        let start_time = app.block_info().time.plus_seconds(1);
        let end_time = app.block_info().time.plus_seconds(10);

        let instantiate_msg = WhitelistInstantiateMsg {
            start_time,
            end_time,
            members: vec![RANDOM.to_string(), RANDOM_2.to_string()],
            unit_price: coin(100, NATIVE_DENOM),
            per_address_limit: 2,
            member_limit: 10,
        };
        let msg = ExecuteMsg::InitWhitelistContract {
            code_id: whitelist_code_id,
            instantiate_msg,
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
        assert_eq!(res.data.whitelist.unwrap(), "contract1")
    }
}

mod actions {
    use super::*;

    mod minting {
        use rift_utils::FundsError;

        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let token_contract_addr = token_contract_instantiation(&mut app);

            let start_time = app.block_info().time.plus_seconds(1);
            let end_time = app.block_info().time.plus_seconds(10);

            setup_whitelist(
                &mut app,
                token_contract_addr.clone(),
                vec![RANDOM.to_string(), RANDOM_2.to_string()],
                start_time,
                end_time,
                coin(100, NATIVE_DENOM),
                2,
            );

            let random_mint = ExecuteMsg::Mint {
                owner: RANDOM.to_string(),
            };
            let random_2_mint = ExecuteMsg::Mint {
                owner: RANDOM_2.to_string(),
            };

            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    token_contract_addr.clone(),
                    &random_mint,
                    &vec![coin(100, NATIVE_DENOM)],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    token_contract_addr.clone(),
                    &random_2_mint,
                    &vec![coin(100, NATIVE_DENOM)],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    token_contract_addr.clone(),
                    &random_mint,
                    &vec![coin(100, NATIVE_DENOM)],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    token_contract_addr.clone(),
                    &random_2_mint,
                    &vec![coin(100, NATIVE_DENOM)],
                )
                .unwrap();

            let token_1_owner = query_token_owner(&app.wrap(), &token_contract_addr, &1).unwrap();
            let token_2_owner = query_token_owner(&app.wrap(), &token_contract_addr, &2).unwrap();
            let token_3_owner = query_token_owner(&app.wrap(), &token_contract_addr, &3).unwrap();
            let token_4_owner = query_token_owner(&app.wrap(), &token_contract_addr, &4).unwrap();

            assert_eq!(token_1_owner, RANDOM.to_string());
            assert_eq!(token_2_owner, RANDOM_2.to_string());
            assert_eq!(token_3_owner, RANDOM.to_string());
            assert_eq!(token_4_owner, RANDOM_2.to_string());
        }

        #[test]
        fn test_invalid_member() {
            let mut app = mock_app();
            let token_contract_addr = token_contract_instantiation(&mut app);

            let start_time = app.block_info().time.plus_seconds(1);
            let end_time = app.block_info().time.plus_seconds(10);

            setup_whitelist(
                &mut app,
                token_contract_addr.clone(),
                vec![RANDOM.to_string(), RANDOM_2.to_string()],
                start_time,
                end_time,
                coin(100, NATIVE_DENOM),
                2,
            );

            app.update_block(|block| block.time = block.time.plus_seconds(5));

            let msg = ExecuteMsg::Mint {
                owner: USER.to_string(),
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    token_contract_addr,
                    &msg,
                    &vec![coin(100, NATIVE_DENOM)],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::NotWhitelisted {}.to_string()
            )
        }

        #[test]
        fn test_token_limit_reached() {
            let mut app = mock_app();
            let token_contract_addr = token_contract_instantiation(&mut app);

            let start_time = app.block_info().time.plus_seconds(1);
            let end_time = app.block_info().time.plus_seconds(10);

            setup_whitelist(
                &mut app,
                token_contract_addr.clone(),
                vec![USER.to_string()],
                start_time,
                end_time,
                coin(100, NATIVE_DENOM),
                2,
            );

            app.update_block(|block| block.time = block.time.plus_seconds(5));

            let msg = ExecuteMsg::Mint {
                owner: USER.to_string(),
            };

            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    token_contract_addr.clone(),
                    &msg,
                    &vec![coin(100, NATIVE_DENOM)],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    token_contract_addr.clone(),
                    &msg,
                    &vec![coin(100, NATIVE_DENOM)],
                )
                .unwrap();

            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    token_contract_addr.clone(),
                    &msg,
                    &vec![coin(100, NATIVE_DENOM)],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::TokenLimitReached {}.to_string()
            )
        }

        #[test]
        fn test_token_price() {
            let mut app = mock_app();
            let token_contract_addr = token_contract_instantiation(&mut app);

            let start_time = app.block_info().time.plus_seconds(1);
            let end_time = app.block_info().time.plus_seconds(10);

            setup_whitelist(
                &mut app,
                token_contract_addr.clone(),
                vec![USER.to_string()],
                start_time,
                end_time,
                coin(100, NATIVE_DENOM),
                2,
            );

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

            app.update_block(|block| block.time = block.time.plus_seconds(5));

            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    token_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                FundsError::MissingFunds {}.to_string()
            );

            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    token_contract_addr.clone(),
                    &msg,
                    &vec![coin(50, NATIVE_DENOM)],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                FundsError::InvalidFunds {
                    got: "50".to_string(),
                    expected: "100".to_string()
                }
                .to_string()
            );
        }
    }
}
