use cosmwasm_std::{coin, Timestamp};
use cosmwasm_std::{Addr, Empty};
use cw721_base::msg::{ExecuteMsg as Cw721ExecuteMsg, QueryMsg as Cw721QueryMsg};
use cw_multi_test::{App, Executor};
use komple_framework_mint_module::msg::ExecuteMsg as MintExecuteMsg;
use komple_framework_token_module::msg::{ExecuteMsg, QueryMsg};
use komple_framework_token_module::ContractError;
use komple_framework_types::modules::token::SubModules as TokenSubModules;
use komple_framework_types::shared::query::ResponseWrapper;
use komple_framework_whitelist_module::msg::InstantiateMsg as WhitelistInstantiateMsg;
use komple_framework_whitelist_module::state::WhitelistConfig;

pub mod helpers;
use helpers::*;

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

mod initialization {
    use super::*;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let (_, token_module_addr) =
            proper_instantiate(&mut app, None, None, None, Some("some-link".to_string()));
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
        assert_eq!(res.data.whitelist.unwrap(), "contract3");

        let res = app.wrap().query_wasm_contract_info("contract3").unwrap();
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
            let (mint_module_addr, token_module_addr) =
                proper_instantiate(&mut app, None, None, None, Some("some-link".to_string()));

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

            let msg = MintExecuteMsg::Mint {
                collection_id: 1,
                metadata_id: None,
            };

            let _ = app
                .execute_contract(
                    Addr::unchecked(USER),
                    mint_module_addr.clone(),
                    &msg,
                    &[coin(100, NATIVE_DENOM)],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(USER),
                    mint_module_addr.clone(),
                    &msg,
                    &[coin(100, NATIVE_DENOM)],
                )
                .unwrap();

            let err = app
                .execute_contract(
                    Addr::unchecked(USER),
                    mint_module_addr,
                    &msg,
                    &[coin(100, NATIVE_DENOM)],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().source().unwrap().to_string(),
                ContractError::TokenLimitReached {}.to_string()
            )
        }
    }
}
