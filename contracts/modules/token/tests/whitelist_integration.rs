use cosmwasm_std::{coin, Timestamp, to_binary};
use cosmwasm_std::{Addr, Coin, Empty, Uint128};
use cw721_base::msg::{ExecuteMsg as Cw721ExecuteMsg, QueryMsg as Cw721QueryMsg};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_metadata_module::msg::InstantiateMsg as MetadataInstantiateMsg;
use komple_token_module::msg::{ExecuteMsg, InstantiateMsg, MetadataInfo, QueryMsg, TokenInfo};
use komple_token_module::state::CollectionConfig;
use komple_token_module::ContractError;
use komple_types::shared::RegisterMsg;
use komple_types::{
    query::ResponseWrapper,
    token::SubModules as TokenSubModules,
};
use komple_types::modules::metadata::Metadata as MetadataType;
use komple_types::modules::mint::Collections;
use komple_whitelist_module::msg::InstantiateMsg as WhitelistInstantiateMsg;
use komple_whitelist_module::state::WhitelistConfig;

pub fn token_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_token_module::contract::execute,
        komple_token_module::contract::instantiate,
        komple_token_module::contract::query,
    )
    .with_reply(komple_token_module::contract::reply);
    Box::new(contract)
}

pub fn metadata_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_metadata_module::contract::execute,
        komple_metadata_module::contract::instantiate,
        komple_metadata_module::contract::query,
    );
    Box::new(contract)
}

pub fn whitelist_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_whitelist_module::contract::execute,
        komple_whitelist_module::contract::instantiate,
        komple_whitelist_module::contract::query,
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
    token_module_addr: Addr,
    members: Vec<String>,
    start_time: Timestamp,
    end_time: Timestamp,
    per_address_limit: u8,
) -> Addr {
    let whitelist_code_id = app.store_code(whitelist_module());

    let instantiate_msg = WhitelistInstantiateMsg {
        members,
        config: WhitelistConfig {
            start_time,
            end_time,
            per_address_limit,
            member_limit: 10,
        },
    };
    let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
        msg: ExecuteMsg::InitWhitelistContract {
            code_id: whitelist_code_id,
            instantiate_msg,
        },
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
        .unwrap();

    let msg = Cw721QueryMsg::Extension {
        msg: QueryMsg::SubModules {},
    };
    let res: ResponseWrapper<TokenSubModules> = app
        .wrap()
        .query_wasm_smart(token_module_addr, &msg)
        .unwrap();

    res.data.whitelist.unwrap()
}

fn token_module_instantiation(app: &mut App) -> Addr {
    let token_code_id = app.store_code(token_module());
    let metadata_code_id = app.store_code(metadata_module());

    let token_info = TokenInfo {
        symbol: "TEST".to_string(),
        minter: ADMIN.to_string(),
    };
    let collection_config = CollectionConfig {
        per_address_limit: None,
        start_time: None,
        max_token_limit: None,
        ipfs_link: Some("some-link".to_string()),
    };
    let metadata_info = MetadataInfo {
        instantiate_msg: MetadataInstantiateMsg {
            metadata_type: MetadataType::Standard,
        },
        code_id: metadata_code_id,
    };
    let msg = InstantiateMsg {
        creator: ADMIN.to_string(),
        token_info,
        collection_type: Collections::Standard,
        collection_name: "Test Collection".to_string(),
        collection_config,
        metadata_info,
    };
    let register_msg = RegisterMsg {
        admin: ADMIN.to_string(),
        data: Some(to_binary(&msg).unwrap()),
    };

    app.instantiate_contract(
        token_code_id,
        Addr::unchecked(ADMIN),
        &register_msg,
        &[],
        "test",
        Some(ADMIN.to_string()),
    )
    .unwrap()
}

mod initialization {
    use super::*;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let token_module_addr = token_module_instantiation(&mut app);
        let whitelist_code_id = app.store_code(whitelist_module());

        let start_time = app.block_info().time.plus_seconds(1);
        let end_time = app.block_info().time.plus_seconds(10);

        let instantiate_msg = WhitelistInstantiateMsg {
            members: vec![RANDOM.to_string(), RANDOM_2.to_string()],
            config: WhitelistConfig {
                start_time,
                end_time,
                per_address_limit: 2,
                member_limit: 10,
            },
        };
        let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
            msg: ExecuteMsg::InitWhitelistContract {
                code_id: whitelist_code_id,
                instantiate_msg,
            },
        };
        let _ = app
            .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
            .unwrap();

        let msg = Cw721QueryMsg::Extension {
            msg: QueryMsg::SubModules {},
        };
        let res: ResponseWrapper<TokenSubModules> = app
            .wrap()
            .query_wasm_smart(token_module_addr, &msg)
            .unwrap();
        assert_eq!(res.data.whitelist.unwrap(), "contract2");

        let res = app.wrap().query_wasm_contract_info("contract2").unwrap();
        assert_eq!(res.admin, Some(ADMIN.to_string()));
    }
}

mod actions {
    use super::*;

    mod minting {
        use super::*;

        #[test]
        fn test_token_limit_reached() {
            let mut app = mock_app();
            let token_module_addr = token_module_instantiation(&mut app);

            let start_time = app.block_info().time.plus_seconds(1);
            let end_time = app.block_info().time.plus_seconds(10);

            setup_whitelist(
                &mut app,
                token_module_addr.clone(),
                vec![USER.to_string()],
                start_time,
                end_time,
                2,
            );

            app.update_block(|block| block.time = block.time.plus_seconds(5));

            let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
                msg: ExecuteMsg::Mint {
                    owner: USER.to_string(),
                    metadata_id: None,
                },
            };

            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    token_module_addr.clone(),
                    &msg,
                    &[coin(100, NATIVE_DENOM)],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    token_module_addr.clone(),
                    &msg,
                    &[coin(100, NATIVE_DENOM)],
                )
                .unwrap();

            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    token_module_addr,
                    &msg,
                    &[coin(100, NATIVE_DENOM)],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::TokenLimitReached {}.to_string()
            )
        }
    }
}
