use cosmwasm_std::{Addr, Coin, Empty, Timestamp, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_framework_metadata_module::msg::InstantiateMsg as MetadataInstantiateMsg;
use komple_framework_mint_module::{
    msg::{CollectionFundInfo, ExecuteMsg as MintExecuteMsg},
    state::CollectionInfo,
};
use komple_framework_token_module::msg::{MetadataInfo, TokenInfo};
use komple_framework_token_module::state::CollectionConfig;
use komple_framework_types::modules::metadata::Metadata as MetadataType;
use komple_framework_types::modules::mint::Collections;
use komple_framework_types::shared::RegisterMsg;
use komple_framework_utils::storage::StorageHelper;

pub fn mint_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_framework_mint_module::contract::execute,
        komple_framework_mint_module::contract::instantiate,
        komple_framework_mint_module::contract::query,
    )
    .with_reply(komple_framework_mint_module::contract::reply);
    Box::new(contract)
}

pub fn token_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_framework_token_module::contract::execute,
        komple_framework_token_module::contract::instantiate,
        komple_framework_token_module::contract::query,
    )
    .with_reply(komple_framework_token_module::contract::reply);
    Box::new(contract)
}

pub fn metadata_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_framework_metadata_module::contract::execute,
        komple_framework_metadata_module::contract::instantiate,
        komple_framework_metadata_module::contract::query,
    );
    Box::new(contract)
}

pub fn whitelist_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_framework_whitelist_module::contract::execute,
        komple_framework_whitelist_module::contract::instantiate,
        komple_framework_whitelist_module::contract::query,
    );
    Box::new(contract)
}

pub const USER: &str = "juno.user";
pub const ADMIN: &str = "juno.admin";
pub const RANDOM: &str = "juno.random";
pub const RANDOM_2: &str = "juno.random2";
pub const NATIVE_DENOM: &str = "denom";
pub const TEST_DENOM: &str = "test_denom";

pub fn mock_app() -> App {
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
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(USER),
                vec![Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(1_000_000),
                }],
            )
            .unwrap();
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(RANDOM),
                vec![Coin {
                    denom: TEST_DENOM.to_string(),
                    amount: Uint128::new(1_000_000),
                }],
            )
            .unwrap();
    })
}

pub fn setup_mint_module(app: &mut App) -> Addr {
    let mint_code_id = app.store_code(mint_module());
    app.instantiate_contract(
        mint_code_id,
        Addr::unchecked(ADMIN),
        &RegisterMsg {
            admin: ADMIN.to_string(),
            data: None,
        },
        &vec![],
        "Mint",
        Some(ADMIN.to_string()),
    )
    .unwrap()
}

pub fn proper_instantiate(
    app: &mut App,
    per_address_limit: Option<u32>,
    start_time: Option<Timestamp>,
    max_token_limit: Option<u32>,
    ipfs_link: Option<String>,
) -> (Addr, Addr) {
    let mint_module_addr = setup_mint_module(app);

    let token_code_id = app.store_code(token_module());
    let metadata_code_id = app.store_code(metadata_module());

    let token_info = TokenInfo {
        symbol: "TTT".to_string(),
        minter: mint_module_addr.to_string(),
    };
    let collection_config = CollectionConfig {
        per_address_limit,
        start_time,
        max_token_limit,
        ipfs_link,
    };
    let metadata_info = MetadataInfo {
        instantiate_msg: MetadataInstantiateMsg {
            metadata_type: MetadataType::Standard,
        },
        code_id: metadata_code_id,
    };
    let create_collection_msg = MintExecuteMsg::CreateCollection {
        code_id: token_code_id,
        collection_info: CollectionInfo {
            collection_type: Collections::Standard,
            name: "Test Collection".to_string(),
            description: "Test Description".to_string(),
            image: "ipfs://xyz".to_string(),
            external_link: None,
        },
        collection_config,
        fund_info: CollectionFundInfo {
            is_native: true,
            denom: NATIVE_DENOM.to_string(),
            cw20_address: None,
        },
        token_info,
        metadata_info,
        linked_collections: None,
    };
    app.execute_contract(
        Addr::unchecked(ADMIN),
        mint_module_addr.clone(),
        &create_collection_msg,
        &vec![],
    )
    .unwrap();

    let token_addr =
        StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();

    (mint_module_addr, token_addr)
}
