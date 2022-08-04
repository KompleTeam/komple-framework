use cosmwasm_std::{Addr, Coin, Empty, Timestamp, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use mint_module::msg::ExecuteMsg as MintExecuteMsg;
use permission_module::msg::ExecuteMsg as PermissionExecuteMsg;
use rift_types::{collection::Collections, module::Modules, permission::Permissions};
use rift_utils::query_module_address;
use token_contract::{
    msg::{ExecuteMsg as TokenExecuteMsg, TokenInfo},
    state::{CollectionInfo, Contracts},
};

use controller_contract::msg::{ExecuteMsg, InstantiateMsg};

pub const USER: &str = "juno..user";
// const RANDOM: &str = "juno..random";
pub const ADMIN: &str = "juno..admin";
pub const NATIVE_DENOM: &str = "denom";

pub fn controller_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        controller_contract::contract::execute,
        controller_contract::contract::instantiate,
        controller_contract::contract::query,
    )
    .with_reply(controller_contract::contract::reply);
    Box::new(contract)
}

pub fn mint_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        mint_module::contract::execute,
        mint_module::contract::instantiate,
        mint_module::contract::query,
    )
    .with_reply(mint_module::contract::reply);
    Box::new(contract)
}

pub fn permission_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        permission_module::contract::execute,
        permission_module::contract::instantiate,
        permission_module::contract::query,
    );
    Box::new(contract)
}

pub fn token_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        token_contract::contract::execute,
        token_contract::contract::instantiate,
        token_contract::contract::query,
    );
    Box::new(contract)
}

pub fn merge_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        merge_module::contract::execute,
        merge_module::contract::instantiate,
        merge_module::contract::query,
    );
    Box::new(contract)
}

pub fn marketplace_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        marketplace_module::contract::execute,
        marketplace_module::contract::instantiate,
        marketplace_module::contract::query,
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
                    amount: Uint128::new(1),
                }],
            )
            .unwrap();
    })
}

pub fn proper_instantiate(app: &mut App) -> Addr {
    let controller_code_id = app.store_code(controller_contract());

    let msg = InstantiateMsg {
        name: "Test Controller".to_string(),
        description: "Test Controller".to_string(),
        image: "https://image.com".to_string(),
        external_link: None,
    };
    let controller_contract_addr = app
        .instantiate_contract(
            controller_code_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "test",
            None,
        )
        .unwrap();

    controller_contract_addr
}

pub fn setup_mint_module(app: &mut App, controller_addr: Addr) {
    let mint_module_code_id = app.store_code(mint_module());

    let msg = ExecuteMsg::InitMintModule {
        code_id: mint_module_code_id,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), controller_addr, &msg, &vec![])
        .unwrap();
}

pub fn setup_merge_module(app: &mut App, controller_addr: Addr) {
    let merge_module_code_id = app.store_code(merge_module());

    let msg = ExecuteMsg::InitMergeModule {
        code_id: merge_module_code_id,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), controller_addr, &msg, &vec![])
        .unwrap();
}

pub fn setup_permission_module(app: &mut App, controller_addr: Addr) {
    let permission_module_code_id = app.store_code(permission_module());

    let msg = ExecuteMsg::InitPermissionModule {
        code_id: permission_module_code_id,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), controller_addr, &msg, &vec![])
        .unwrap();
}

pub fn setup_marketplace_module(app: &mut App, controller_addr: Addr) {
    let marketplace_module_code_id = app.store_code(marketplace_module());

    let msg = ExecuteMsg::InitMarketplaceModule {
        code_id: marketplace_module_code_id,
        native_denom: NATIVE_DENOM.to_string(),
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), controller_addr, &msg, &vec![])
        .unwrap();
}

pub fn setup_all_modules(app: &mut App, controller_addr: Addr) {
    setup_mint_module(app, controller_addr.clone());
    setup_merge_module(app, controller_addr.clone());
    setup_permission_module(app, controller_addr.clone());
    setup_marketplace_module(app, controller_addr.clone());
}

pub fn create_collection(
    app: &mut App,
    mint_module_addr: Addr,
    token_contract_code_id: u64,
    per_address_limit: Option<u32>,
    start_time: Option<Timestamp>,
    collection_type: Collections,
    linked_collections: Option<Vec<u32>>,
    contracts: Contracts,
) {
    let collection_info = CollectionInfo {
        collection_type,
        name: "Test Collection".to_string(),
        description: "Test Collection".to_string(),
        image: "https://image.com".to_string(),
        external_link: None,
    };
    let token_info = TokenInfo {
        symbol: "TEST".to_string(),
        minter: mint_module_addr.to_string(),
    };
    let msg = MintExecuteMsg::CreateCollection {
        code_id: token_contract_code_id,
        collection_info,
        token_info,
        per_address_limit,
        start_time,
        linked_collections,
        contracts,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), mint_module_addr, &msg, &vec![])
        .unwrap();
}

pub fn mint_token(app: &mut App, mint_module_addr: Addr, collection_id: u32, sender: &str) {
    let msg = MintExecuteMsg::Mint { collection_id };
    let _ = app
        .execute_contract(Addr::unchecked(sender), mint_module_addr, &msg, &vec![])
        .unwrap();
}

pub fn setup_mint_module_whitelist(app: &mut App, mint_module_addr: Addr, addrs: Vec<String>) {
    let msg = MintExecuteMsg::UpdateWhitelistAddresses { addrs };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), mint_module_addr, &msg, &vec![])
        .unwrap();
}

pub fn setup_token_contract_operators(
    app: &mut App,
    token_contract_addr: Addr,
    addrs: Vec<String>,
) {
    let msg = TokenExecuteMsg::UpdateOperators { addrs };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), token_contract_addr, &msg, &vec![])
        .unwrap();
}

pub fn give_approval_to_module(
    app: &mut App,
    token_contract_addr: Addr,
    owner: &str,
    operator_addr: &Addr,
) {
    let msg = TokenExecuteMsg::ApproveAll {
        operator: operator_addr.to_string(),
        expires: None,
    };
    let _ = app
        .execute_contract(Addr::unchecked(owner), token_contract_addr, &msg, &vec![])
        .unwrap();
}

pub fn add_permission_for_module(
    app: &mut App,
    permission_module_addr: Addr,
    module: Modules,
    permissions: Vec<Permissions>,
) {
    let msg = PermissionExecuteMsg::UpdateModulePermissions {
        module,
        permissions,
    };
    let _ = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            permission_module_addr,
            &msg,
            &vec![],
        )
        .unwrap();
}

pub fn link_collection_to_collections(
    app: &mut App,
    mint_module_addr: Addr,
    collection_id: u32,
    linked_collections: Vec<u32>,
) {
    let msg = MintExecuteMsg::UpdateLinkedCollections {
        collection_id,
        linked_collections,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), mint_module_addr, &msg, &vec![])
        .unwrap();
}

pub fn get_modules_addresses(app: &mut App, controller_addr: &Addr) -> (Addr, Addr, Addr, Addr) {
    let mint_module_addr: Addr;
    let merge_module_addr: Addr;
    let permission_module_addr: Addr;
    let marketplace_module_addr: Addr;

    let res = query_module_address(&app.wrap(), controller_addr, Modules::MintModule);
    mint_module_addr = res.unwrap();

    let res = query_module_address(&app.wrap(), controller_addr, Modules::MergeModule);
    merge_module_addr = res.unwrap();

    let res = query_module_address(&app.wrap(), controller_addr, Modules::PermissionModule);
    permission_module_addr = res.unwrap();

    let res = query_module_address(&app.wrap(), controller_addr, Modules::MarketplaceModule);
    marketplace_module_addr = res.unwrap();

    // println!("");
    // println!("mint_module_addr: {}", mint_module_addr);
    // println!("merge_module_addr: {}", merge_module_addr);
    // println!("permission_module_addr: {}", permission_module_addr);
    // println!("");

    (
        mint_module_addr,
        merge_module_addr,
        permission_module_addr,
        marketplace_module_addr,
    )
}
