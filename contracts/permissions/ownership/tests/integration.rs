use cosmwasm_std::{coin, to_binary, Addr, Coin, Empty, Uint128};
use cw721::OwnerOfResponse;
use cw721_base::msg::QueryMsg as Cw721QueryMsg;
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_framework_hub_module::msg::{
    ExecuteMsg as HubExecuteMsg, InstantiateMsg as HubInstantiateMsg, QueryMsg as HubQueryMsg,
};
use komple_framework_hub_module::state::HubInfo;
use komple_framework_metadata_module::msg::InstantiateMsg as MetadataInstantiateMsg;
use komple_framework_mint_module::msg::{CollectionFundInfo, ExecuteMsg as MintExecuteMsg};
use komple_framework_mint_module::state::CollectionInfo;
use komple_framework_ownership_permission::msg::OwnershipMsg;
use komple_framework_permission_module::msg::{
    ExecuteMsg as PermissionExecuteMsg, PermissionCheckMsg,
};
use komple_framework_permission_module::ContractError;
use komple_framework_token_module::msg::{MetadataInfo, QueryMsg as TokenQueryMsg, TokenInfo};
use komple_framework_token_module::state::CollectionConfig;
use komple_framework_types::modules::metadata::Metadata as MetadataType;
use komple_framework_types::modules::mint::Collections;
use komple_framework_types::modules::permission::Permissions;
use komple_framework_types::modules::Modules;
use komple_framework_types::shared::query::ResponseWrapper;
use komple_framework_types::shared::RegisterMsg;
use komple_framework_utils::storage::StorageHelper;

pub const USER: &str = "juno..user";
pub const RANDOM: &str = "juno..random";
pub const ADMIN: &str = "juno..admin";
pub const NATIVE_DENOM: &str = "native_denom";
pub const TEST_DENOM: &str = "test_denom";

pub fn hub_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_framework_hub_module::contract::execute,
        komple_framework_hub_module::contract::instantiate,
        komple_framework_hub_module::contract::query,
    )
    .with_reply(komple_framework_hub_module::contract::reply);
    Box::new(contract)
}

pub fn mint_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_framework_mint_module::contract::execute,
        komple_framework_mint_module::contract::instantiate,
        komple_framework_mint_module::contract::query,
    )
    .with_reply(komple_framework_mint_module::contract::reply);
    Box::new(contract)
}

pub fn permission_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_framework_permission_module::contract::execute,
        komple_framework_permission_module::contract::instantiate,
        komple_framework_permission_module::contract::query,
    )
    .with_reply(komple_framework_permission_module::contract::reply);
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

pub fn ownership_permission_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_framework_ownership_permission::contract::execute,
        komple_framework_ownership_permission::contract::instantiate,
        komple_framework_ownership_permission::contract::query,
    );
    Box::new(contract)
}

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
    })
}

fn setup_hub_module(app: &mut App) -> Addr {
    let hub_code_id = app.store_code(hub_module());

    let msg = HubInstantiateMsg {
        hub_info: HubInfo {
            name: "Test Hub".to_string(),
            description: "Test Hub".to_string(),
            image: "https://example.com/image.png".to_string(),
            external_link: None,
        },
        marbu_fee_module: None,
    };
    let register_msg = RegisterMsg {
        admin: ADMIN.to_string(),
        data: Some(to_binary(&msg).unwrap()),
    };

    app.instantiate_contract(
        hub_code_id,
        Addr::unchecked(ADMIN),
        &register_msg,
        &[coin(1_000_000, NATIVE_DENOM)],
        "test",
        None,
    )
    .unwrap()
}

fn setup_modules(app: &mut App, hub_addr: Addr) -> (Addr, Addr) {
    let mint_code_id = app.store_code(mint_module());
    let permission_code_id = app.store_code(permission_module());

    let instantiate_msg = to_binary(&RegisterMsg {
        admin: ADMIN.to_string(),
        data: None,
    })
    .unwrap();
    let msg = HubExecuteMsg::RegisterModule {
        module: Modules::Mint.to_string(),
        msg: Some(instantiate_msg),
        code_id: mint_code_id,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &[])
        .unwrap();
    let instantiate_msg = to_binary(&RegisterMsg {
        admin: ADMIN.to_string(),
        data: None,
    })
    .unwrap();
    let msg = HubExecuteMsg::RegisterModule {
        module: Modules::Permission.to_string(),
        msg: Some(instantiate_msg),
        code_id: permission_code_id,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &[])
        .unwrap();

    let msg = HubQueryMsg::ModuleAddress {
        module: Modules::Mint.to_string(),
    };
    let mint_res: ResponseWrapper<Addr> =
        app.wrap().query_wasm_smart(hub_addr.clone(), &msg).unwrap();
    let msg = HubQueryMsg::ModuleAddress {
        module: Modules::Permission.to_string(),
    };
    let permission_res: ResponseWrapper<Addr> =
        app.wrap().query_wasm_smart(hub_addr, &msg).unwrap();

    (mint_res.data, permission_res.data)
}

pub fn create_collection(app: &mut App, mint_module_addr: Addr, token_module_code_id: u64) {
    let metadata_code_id = app.store_code(metadata_module());

    let collection_info = CollectionInfo {
        collection_type: Collections::Standard,
        name: "Test Collection".to_string(),
        description: "Test Collection".to_string(),
        image: "https://image.com".to_string(),
        external_link: None,
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
    let fund_info = CollectionFundInfo {
        is_native: true,
        denom: NATIVE_DENOM.to_string(),
        cw20_address: None,
    };
    let msg = MintExecuteMsg::CreateCollection {
        code_id: token_module_code_id,
        collection_config,
        collection_info,
        metadata_info,
        token_info,
        fund_info,
        linked_collections: None,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), mint_module_addr, &msg, &[])
        .unwrap();
}

pub fn mint_token(app: &mut App, mint_module_addr: Addr, collection_id: u32, sender: &str) {
    let msg = MintExecuteMsg::Mint {
        collection_id,
        metadata_id: None,
    };
    let _ = app
        .execute_contract(Addr::unchecked(sender), mint_module_addr, &msg, &[])
        .unwrap();
}

fn setup_ownership_permission_module(app: &mut App) -> Addr {
    let ownership_permission_code_id = app.store_code(ownership_permission_module());

    let msg = RegisterMsg {
        admin: ADMIN.to_string(),
        data: None,
    };

    app.instantiate_contract(
        ownership_permission_code_id,
        Addr::unchecked(ADMIN),
        &msg,
        &[],
        "test",
        None,
    )
    .unwrap()
}

fn setup_module_permissions(
    app: &mut App,
    permission_module_addr: &Addr,
    module: String,
    permissions: Vec<String>,
) {
    let msg = PermissionExecuteMsg::UpdateModulePermissions {
        module,
        permissions,
    };
    let _ = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            permission_module_addr.clone(),
            &msg,
            &[],
        )
        .unwrap();
}

fn register_permission(app: &mut App, permission_module_addr: &Addr) {
    let ownership_permission_code_id = app.store_code(ownership_permission_module());

    let msg = PermissionExecuteMsg::RegisterPermission {
        permission: Permissions::Ownership.to_string(),
        msg: Some(
            to_binary(&RegisterMsg {
                admin: ADMIN.to_string(),
                data: None,
            })
            .unwrap(),
        ),
        code_id: ownership_permission_code_id,
    };
    let _ = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            permission_module_addr.clone(),
            &msg,
            &[],
        )
        .unwrap();
}

#[test]
fn test_update_module_permissions() {
    let mut app = mock_app();
    let hub_addr = setup_hub_module(&mut app);
    let (_, permission_module_addr) = setup_modules(&mut app, hub_addr);

    setup_ownership_permission_module(&mut app);
    register_permission(&mut app, &permission_module_addr);

    let msg = PermissionExecuteMsg::UpdateModulePermissions {
        module: Modules::Mint.to_string(),
        permissions: vec![Permissions::Attribute.to_string()],
    };
    let err = app
        .execute_contract(
            Addr::unchecked(USER),
            permission_module_addr.clone(),
            &msg,
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::Unauthorized {}.to_string()
    );

    let msg = PermissionExecuteMsg::UpdateModulePermissions {
        module: Modules::Mint.to_string(),
        permissions: vec![Permissions::Ownership.to_string()],
    };
    let _ = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            permission_module_addr.clone(),
            &msg,
            &[],
        )
        .unwrap();

    let msg = PermissionExecuteMsg::UpdateModulePermissions {
        module: Modules::Mint.to_string(),
        permissions: vec![Permissions::Attribute.to_string()],
    };
    let err = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            permission_module_addr.clone(),
            &msg,
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::InvalidPermissions {}.to_string()
    )
}

#[test]
fn test_permission_check() {
    let mut app = mock_app();
    let hub_addr = setup_hub_module(&mut app);
    let (mint_module_addr, permission_module_addr) = setup_modules(&mut app, hub_addr);
    let token_module_code_id = app.store_code(token_module());
    create_collection(&mut app, mint_module_addr.clone(), token_module_code_id);
    let collection_addr =
        StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();
    mint_token(&mut app, mint_module_addr, 1, USER);
    setup_ownership_permission_module(&mut app);
    register_permission(&mut app, &permission_module_addr);
    setup_module_permissions(
        &mut app,
        &permission_module_addr,
        Modules::Mint.to_string(),
        vec![Permissions::Ownership.to_string()],
    );

    // Make sure token owner is USER
    let msg: Cw721QueryMsg<TokenQueryMsg> = Cw721QueryMsg::OwnerOf {
        token_id: "1".to_string(),
        include_expired: None,
    };
    let res: OwnerOfResponse = app.wrap().query_wasm_smart(collection_addr, &msg).unwrap();
    assert_eq!(res.owner, USER);

    let msg = PermissionExecuteMsg::Check {
        module: Modules::Mint.to_string(),
        msg: to_binary(&[PermissionCheckMsg {
            permission_type: Permissions::Ownership.to_string(),
            data: to_binary(&[OwnershipMsg {
                collection_id: 1,
                token_id: 1,
                address: USER.to_string(),
            }])
            .unwrap(),
        }])
        .unwrap(),
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), permission_module_addr, &msg, &[])
        .unwrap();
}
