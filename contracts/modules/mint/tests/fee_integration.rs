use cosmwasm_std::{to_binary, Addr, Coin, Empty, Uint128};
use cw721_base::msg::ExecuteMsg as Cw721ExecuteMsg;
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_fee_module::msg::ExecuteMsg as FeeExecuteMsg;
use komple_hub_module::msg::{ExecuteMsg as HubExecuteMsg, InstantiateMsg as HubInstantiateMsg};
use komple_hub_module::state::HubInfo;
use komple_metadata_module::msg::InstantiateMsg as MetadataInstantiateMsg;
use komple_mint_module::msg::ExecuteMsg;
use komple_mint_module::state::CollectionInfo;
use komple_mint_module::ContractError;
use komple_token_module::msg::{ExecuteMsg as TokenExecuteMsg, MetadataInfo, TokenInfo};
use komple_token_module::state::CollectionConfig;
use komple_types::collection::Collections;
use komple_types::fee::MintFees;
use komple_types::fee::{Fees, FixedPayment};
use komple_types::metadata::Metadata as MetadataType;
use komple_types::module::Modules;
use komple_types::shared::RegisterMsg;
use komple_utils::storage::StorageHelper;
use komple_whitelist_module::msg::InstantiateMsg as WhitelistInstantiateMsg;
use komple_whitelist_module::state::WhitelistConfig;

pub const USER: &str = "juno..user";
pub const USER2: &str = "juno..user2";
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

pub fn whitelist_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_whitelist_module::contract::execute,
        komple_whitelist_module::contract::instantiate,
        komple_whitelist_module::contract::query,
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
                    amount: Uint128::new(1000),
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

pub fn register_module(app: &mut App, hub_addr: &Addr, module: String, code_id: u64) {
    app.execute_contract(
        Addr::unchecked(ADMIN),
        hub_addr.clone(),
        &HubExecuteMsg::RegisterModule {
            module: module.to_string(),
            msg: Some(
                to_binary(&RegisterMsg {
                    admin: ADMIN.to_string(),
                    data: None,
                })
                .unwrap(),
            ),
            code_id,
        },
        &vec![],
    )
    .unwrap();
}

pub fn create_collection(app: &mut App, mint_module_addr: &Addr) {
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
}

pub fn set_fixed_fee(
    app: &mut App,
    fee_module_addr: &Addr,
    fee_name: &str,
    collection_id: u32,
    value: u128,
) {
    app.execute_contract(
        Addr::unchecked(ADMIN),
        fee_module_addr.clone(),
        &FeeExecuteMsg::SetFee {
            fee_type: Fees::Fixed,
            module_name: Modules::Mint.to_string(),
            fee_name: format!("{}/{}", fee_name, collection_id),
            data: to_binary(&FixedPayment {
                address: Some(ADMIN.to_string()),
                value: Uint128::new(value),
            })
            .unwrap(),
        },
        &[],
    )
    .unwrap();
}

pub fn create_whitelist(app: &mut App, collection_addr: &Addr) {
    let whitelist_code_id = app.store_code(whitelist_module());
    let msg: Cw721ExecuteMsg<Empty, TokenExecuteMsg> = Cw721ExecuteMsg::Extension {
        msg: TokenExecuteMsg::InitWhitelistContract {
            code_id: whitelist_code_id,
            instantiate_msg: WhitelistInstantiateMsg {
                members: vec![USER.to_string()],
                config: WhitelistConfig {
                    start_time: app.block_info().time.plus_seconds(1),
                    end_time: app.block_info().time.plus_seconds(50),
                    per_address_limit: 5,
                    member_limit: 10,
                },
            },
        },
    };
    app.execute_contract(Addr::unchecked(ADMIN), collection_addr.clone(), &msg, &[])
        .unwrap();
}

mod execute {
    use super::*;

    mod mint {
        use super::*;

        #[test]
        fn test_standard_price() {
            let mut app = mock_app();
            let hub_addr = setup_hub_module(&mut app);

            // Register Mint Module
            let mint_code_id = app.store_code(mint_module());
            register_module(&mut app, &hub_addr, Modules::Mint.to_string(), mint_code_id);
            let mint_module_addr =
                StorageHelper::query_module_address(&app.wrap(), &hub_addr, Modules::Mint).unwrap();

            // Register fee module
            let fee_code_id = app.store_code(fee_module());
            register_module(&mut app, &hub_addr, Modules::Fee.to_string(), fee_code_id);
            let fee_module_addr =
                StorageHelper::query_module_address(&app.wrap(), &hub_addr, Modules::Fee).unwrap();

            // Create collection
            create_collection(&mut app, &mint_module_addr);

            // Set mint fee
            set_fixed_fee(&mut app, &fee_module_addr, MintFees::Price.as_str(), 1, 10);

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

        #[test]
        fn test_whitelist_price() {
            let mut app = mock_app();
            let hub_addr = setup_hub_module(&mut app);

            // Register Mint Module
            let mint_code_id = app.store_code(mint_module());
            register_module(&mut app, &hub_addr, Modules::Mint.to_string(), mint_code_id);
            let mint_module_addr =
                StorageHelper::query_module_address(&app.wrap(), &hub_addr, Modules::Mint).unwrap();

            // Register fee module
            let fee_code_id = app.store_code(fee_module());
            register_module(&mut app, &hub_addr, Modules::Fee.to_string(), fee_code_id);
            let fee_module_addr =
                StorageHelper::query_module_address(&app.wrap(), &hub_addr, Modules::Fee).unwrap();

            // Create collection
            create_collection(&mut app, &mint_module_addr);
            let collection_addr =
                StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &1)
                    .unwrap();

            // Create whitelist
            create_whitelist(&mut app, &collection_addr);

            // Set whitelist fee
            set_fixed_fee(
                &mut app,
                &fee_module_addr,
                MintFees::Whitelist.as_str(),
                1,
                10,
            );

            app.update_block(|block| {
                block.time = block.time.plus_seconds(2);
            });

            // Throw error if invalid fund
            let err = app
                .execute_contract(
                    Addr::unchecked(USER2),
                    mint_module_addr.clone(),
                    &ExecuteMsg::Mint {
                        collection_id: 1,
                        metadata_id: None,
                    },
                    &[],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AddressNotWhitelisted {}.to_string()
            );

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

            // Success
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

            app.update_block(|block| block.time = block.time.plus_seconds(100));

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
