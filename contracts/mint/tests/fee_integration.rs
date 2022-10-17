use cosmwasm_std::{to_binary, Addr, Coin, Empty, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_fee_module::msg::{ExecuteMsg as FeeExecuteMsg, InstantiateMsg as FeeInstantiateMsg};
use komple_hub_module::msg::{ExecuteMsg as HubExecuteMsg, InstantiateMsg as HubInstantiateMsg};
use komple_hub_module::state::HubInfo;
use komple_metadata_module::msg::InstantiateMsg as MetadataInstantiateMsg;
use komple_mint_module::msg::{ExecuteMsg, InstantiateMsg};
use komple_mint_module::state::CollectionInfo;
use komple_token_module::msg::{MetadataInfo, TokenInfo};
use komple_token_module::state::CollectionConfig;
use komple_types::collection::Collections;
use komple_types::fee::{Fees, FixedPayment};
use komple_types::metadata::Metadata as MetadataType;
use komple_types::module::Modules;
use komple_utils::storage::StorageHelper;

pub const USER: &str = "juno..user";
pub const ADMIN: &str = "juno..admin";
pub const NATIVE_DENOM: &str = "native_denom";

pub fn hub_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_hub_module::contract::execute,
        komple_hub_module::contract::instantiate,
        komple_hub_module::contract::query,
    )
    .with_reply(komple_hub_module::contract::reply);
    Box::new(contract)
}

pub fn mint_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_mint_module::contract::execute,
        komple_mint_module::contract::instantiate,
        komple_mint_module::contract::query,
    )
    .with_reply(komple_mint_module::contract::reply);
    Box::new(contract)
}

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

pub fn fee_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_fee_module::contract::execute,
        komple_fee_module::contract::instantiate,
        komple_fee_module::contract::query,
    );
    Box::new(contract)
}

pub fn mock_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(USER),
                vec![Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(10),
                }],
            )
            .unwrap();
    })
}

fn setup_hub_module(app: &mut App) -> Addr {
    let hub_code_id = app.store_code(hub_module());

    let msg = HubInstantiateMsg {
        admin: None,
        hub_info: HubInfo {
            name: "Test Hub".to_string(),
            description: "Test Hub".to_string(),
            image: "https://example.com/image.png".to_string(),
            external_link: None,
        },
        marbu_fee_module: None,
    };

    app.instantiate_contract(hub_code_id, Addr::unchecked(ADMIN), &msg, &[], "test", None)
        .unwrap()
}

mod execute {
    use super::*;

    mod mint {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let hub_addr = setup_hub_module(&mut app);

            // Register Mint Module
            let mint_code_id = app.store_code(mint_module());
            app.execute_contract(
                Addr::unchecked(ADMIN),
                hub_addr.clone(),
                &HubExecuteMsg::RegisterModule {
                    module: Modules::Mint.to_string(),
                    msg: to_binary(&InstantiateMsg {
                        admin: ADMIN.to_string(),
                    })
                    .unwrap(),
                    code_id: mint_code_id,
                },
                &vec![],
            )
            .unwrap();
            let mint_module_addr =
                StorageHelper::query_module_address(&app.wrap(), &hub_addr, Modules::Mint).unwrap();

            // Register fee module
            let fee_code_id = app.store_code(fee_module());
            app.execute_contract(
                Addr::unchecked(ADMIN),
                hub_addr.clone(),
                &HubExecuteMsg::RegisterModule {
                    module: Modules::Fee.to_string(),
                    msg: to_binary(&FeeInstantiateMsg {
                        admin: ADMIN.to_string(),
                    })
                    .unwrap(),
                    code_id: fee_code_id,
                },
                &vec![],
            )
            .unwrap();
            let fee_module_addr =
                StorageHelper::query_module_address(&app.wrap(), &hub_addr, Modules::Fee).unwrap();

            // Create collection
            let token_code_id = app.store_code(token_module());
            let metadata_code_id = app.store_code(metadata_module());
            let collection_info = CollectionInfo {
                collection_type: Collections::Standard,
                name: "Test Collection".to_string(),
                description: "Test Collection".to_string(),
                image: "https://image.com".to_string(),
                external_link: None,
                native_denom: NATIVE_DENOM.to_string(),
            };
            let token_info = TokenInfo {
                symbol: "TEST".to_string(),
                minter: mint_module_addr.to_string(),
            };
            let collection_config = CollectionConfig {
                per_address_limit: None,
                start_time: None,
                max_token_limit: None,
                ipfs_link: Some("some-link".to_string()),
            };
            let metadata_info = MetadataInfo {
                instantiate_msg: MetadataInstantiateMsg {
                    admin: "".to_string(),
                    metadata_type: MetadataType::Standard,
                },
                code_id: metadata_code_id,
            };
            app.execute_contract(
                Addr::unchecked(ADMIN),
                mint_module_addr.clone(),
                &ExecuteMsg::CreateCollection {
                    code_id: token_code_id,
                    collection_config,
                    collection_info,
                    metadata_info,
                    token_info,
                    linked_collections: None,
                },
                &[],
            )
            .unwrap();

            // Set mint fee
            app.execute_contract(
                Addr::unchecked(ADMIN),
                fee_module_addr.clone(),
                &FeeExecuteMsg::SetFee {
                    fee_type: Fees::Fixed,
                    module_name: Modules::Mint.to_string(),
                    fee_name: "collection_1".to_string(),
                    data: to_binary(&FixedPayment {
                        address: Some(ADMIN.to_string()),
                        value: Uint128::new(10),
                    })
                    .unwrap(),
                },
                &[],
            )
            .unwrap();

            // Throw error if invalid fund
            app.execute_contract(
                Addr::unchecked(USER),
                mint_module_addr.clone(),
                &ExecuteMsg::Mint {
                    collection_id: 1,
                    metadata_id: None,
                },
                &[Coin {
                    amount: Uint128::new(5),
                    denom: NATIVE_DENOM.to_string(),
                }],
            )
            .unwrap_err();

            app.execute_contract(
                Addr::unchecked(USER),
                mint_module_addr.clone(),
                &ExecuteMsg::Mint {
                    collection_id: 1,
                    metadata_id: None,
                },
                &[Coin {
                    amount: Uint128::new(10),
                    denom: NATIVE_DENOM.to_string(),
                }],
            )
            .unwrap();

            let res = app.wrap().query_balance(ADMIN, NATIVE_DENOM).unwrap();
            assert_eq!(res.amount, Uint128::new(10));
        }
    }
}
