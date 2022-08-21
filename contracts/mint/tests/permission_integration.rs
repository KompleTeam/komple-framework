use cosmwasm_std::{Addr, Coin, Decimal, Empty, Timestamp, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use hub_contract::msg::{
    ExecuteMsg as HubExecuteMsg, InstantiateMsg as HubInstantiateMsg, QueryMsg as HubQueryMsg,
};
use komple_token_module::{
    msg::{
        ExecuteMsg as TokenExecuteMsg, InstantiateMsg as TokenInstantiateMsg,
        QueryMsg as TokenQueryMsg, TokenInfo,
    },
    state::{BundleInfo, Contracts},
};
use komple_types::bundle::Bundles;
use komple_types::metadata::Metadata as MetadataType;
use komple_types::module::Modules;
use komple_types::query::ResponseWrapper;
use metadata_contract::msg::ExecuteMsg as MetadataExecuteMsg;
use metadata_contract::state::{MetaInfo, Trait};
use mint_module::msg::ExecuteMsg;
use permission_module::msg::ExecuteMsg as PermissionExecuteMsg;

pub const USER: &str = "juno..user";
pub const RANDOM: &str = "juno..random";
pub const ADMIN: &str = "juno..admin";
pub const NATIVE_DENOM: &str = "denom";
pub const TEST_DENOM: &str = "test_denom";

pub fn hub_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        hub_contract::contract::execute,
        hub_contract::contract::instantiate,
        hub_contract::contract::query,
    )
    .with_reply(hub_contract::contract::reply);
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

pub fn token_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_token_module::contract::execute,
        komple_token_module::contract::instantiate,
        komple_token_module::contract::query,
    )
    .with_reply(komple_token_module::contract::reply);
    Box::new(contract)
}

pub fn metadata_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        metadata_contract::contract::execute,
        metadata_contract::contract::instantiate,
        metadata_contract::contract::query,
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
                    amount: Uint128::new(1_000_000),
                }],
            )
            .unwrap();
    })
}

fn setup_hub_contract(app: &mut App) -> Addr {
    let hub_code_id = app.store_code(hub_contract());

    let msg = HubInstantiateMsg {
        name: "Test Hub".to_string(),
        description: "Test Hub".to_string(),
        image: "https://example.com/image.png".to_string(),
        external_link: None,
    };
    let hub_addr = app
        .instantiate_contract(
            hub_code_id,
            Addr::unchecked(ADMIN),
            &msg,
            &vec![],
            "test",
            None,
        )
        .unwrap();

    hub_addr
}

fn setup_modules(app: &mut App, hub_addr: Addr) -> (Addr, Addr) {
    let mint_code_id = app.store_code(mint_module());
    let permission_code_id = app.store_code(permission_module());

    let msg = HubExecuteMsg::InitMintModule {
        code_id: mint_code_id,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &vec![])
        .unwrap();
    let msg = HubExecuteMsg::InitPermissionModule {
        code_id: permission_code_id,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &vec![])
        .unwrap();

    let msg = HubQueryMsg::ModuleAddress(Modules::Mint);
    let mint_res: ResponseWrapper<Addr> =
        app.wrap().query_wasm_smart(hub_addr.clone(), &msg).unwrap();
    let msg = HubQueryMsg::ModuleAddress(Modules::Permission);
    let permission_res: ResponseWrapper<Addr> =
        app.wrap().query_wasm_smart(hub_addr.clone(), &msg).unwrap();

    (mint_res.data, permission_res.data)
}

pub fn create_bundle(
    app: &mut App,
    mint_module_addr: Addr,
    token_module_code_id: u64,
    per_address_limit: Option<u32>,
    start_time: Option<Timestamp>,
    bundle_type: Bundles,
    linked_bundles: Option<Vec<u32>>,
    unit_price: Option<Uint128>,
    max_token_limit: Option<u32>,
    royalty_share: Option<Decimal>,
) {
    let bundle_info = BundleInfo {
        bundle_type,
        name: "Test Bundle".to_string(),
        description: "Test Bundle".to_string(),
        image: "https://image.com".to_string(),
        external_link: None,
    };
    let token_info = TokenInfo {
        symbol: "TEST".to_string(),
        minter: mint_module_addr.to_string(),
    };
    let msg = ExecuteMsg::CreateBundle {
        code_id: token_module_code_id,
        token_instantiate_msg: TokenInstantiateMsg {
            admin: ADMIN.to_string(),
            bundle_info,
            token_info,
            per_address_limit,
            start_time,
            unit_price,
            native_denom: NATIVE_DENOM.to_string(),
            max_token_limit,
            royalty_share,
        },
        linked_bundles,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), mint_module_addr, &msg, &vec![])
        .unwrap();
}

pub fn setup_metadata_contract(
    app: &mut App,
    token_module_addr: Addr,
    metadata_type: MetadataType,
) -> Addr {
    let metadata_code_id = app.store_code(metadata_contract());

    let msg = TokenExecuteMsg::InitMetadataContract {
        code_id: metadata_code_id,
        metadata_type,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), token_module_addr.clone(), &msg, &[])
        .unwrap();

    let res: ResponseWrapper<Contracts> = app
        .wrap()
        .query_wasm_smart(token_module_addr.clone(), &TokenQueryMsg::Contracts {})
        .unwrap();
    res.data.metadata.unwrap()
}

pub fn setup_metadata(app: &mut App, metadata_contract_addr: Addr) {
    let meta_info = MetaInfo {
        image: Some("https://some-image.com".to_string()),
        external_url: None,
        description: Some("Some description".to_string()),
        youtube_url: None,
        animation_url: None,
    };
    let attributes = vec![
        Trait {
            trait_type: "trait_1".to_string(),
            value: "value_1".to_string(),
        },
        Trait {
            trait_type: "trait_2".to_string(),
            value: "value_2".to_string(),
        },
    ];
    let msg = MetadataExecuteMsg::AddMetadata {
        meta_info,
        attributes,
    };
    let _ = app
        .execute_contract(
            Addr::unchecked(ADMIN),
            metadata_contract_addr.clone(),
            &msg,
            &vec![],
        )
        .unwrap();
}

pub fn mint_token(app: &mut App, mint_module_addr: Addr, bundle_id: u32, sender: &str) {
    let msg = ExecuteMsg::Mint {
        bundle_id,
        metadata_id: None,
    };
    let _ = app
        .execute_contract(Addr::unchecked(sender), mint_module_addr, &msg, &vec![])
        .unwrap();
}

mod initialization {
    use super::*;

    use komple_types::module::Modules;

    use hub_contract::ContractError;
    use komple_utils::query_module_address;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let hub_addr = setup_hub_contract(&mut app);
        let mint_module_code_id = app.store_code(mint_module());

        let msg = HubExecuteMsg::InitMintModule {
            code_id: mint_module_code_id,
        };
        let _ = app.execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &vec![]);

        let res = query_module_address(&app.wrap(), &hub_addr, Modules::Mint).unwrap();
        assert_eq!(res, "contract1")
    }

    #[test]
    fn test_invalid_sender() {
        let mut app = mock_app();
        let hub_addr = setup_hub_contract(&mut app);
        let mint_module_code_id = app.store_code(mint_module());

        let msg = HubExecuteMsg::InitMergeModule {
            code_id: mint_module_code_id,
        };
        let err = app
            .execute_contract(Addr::unchecked(USER), hub_addr.clone(), &msg, &vec![])
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            ContractError::Unauthorized {}.to_string()
        )
    }
}

mod permission_mint {
    use komple_utils::query_bundle_address;

    use super::*;

    use cosmwasm_std::to_binary;
    use cw721::OwnerOfResponse;
    use komple_token_module::msg::QueryMsg as TokenQueryMsg;
    use komple_types::{
        bundle::Bundles, metadata::Metadata, module::Modules, permission::Permissions,
    };
    use mint_module::msg::ExecuteMsg as MintExecuteMsg;
    use permission_module::msg::{OwnershipMsg, PermissionCheckMsg};

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let hub_addr = setup_hub_contract(&mut app);

        let (mint_module_addr, permission_module_addr) = setup_modules(&mut app, hub_addr.clone());

        let token_module_code_id = app.store_code(token_module());
        create_bundle(
            &mut app,
            mint_module_addr.clone(),
            token_module_code_id,
            None,
            None,
            Bundles::Normal,
            None,
            None,
            None,
            None,
        );
        create_bundle(
            &mut app,
            mint_module_addr.clone(),
            token_module_code_id,
            None,
            None,
            Bundles::Normal,
            None,
            None,
            None,
            None,
        );

        let bundle_addr_1 =
            query_bundle_address(&app.wrap(), &mint_module_addr.clone(), &1).unwrap();
        let bundle_addr_2 =
            query_bundle_address(&app.wrap(), &mint_module_addr.clone(), &2).unwrap();

        let metadata_contract_addr_1 =
            setup_metadata_contract(&mut app, bundle_addr_1, Metadata::Standard);
        let metadata_contract_addr_2 =
            setup_metadata_contract(&mut app, bundle_addr_2, Metadata::Standard);

        setup_metadata(&mut app, metadata_contract_addr_1.clone());
        setup_metadata(&mut app, metadata_contract_addr_1.clone());
        setup_metadata(&mut app, metadata_contract_addr_2);

        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 1, USER);

        let msg = PermissionExecuteMsg::UpdateModulePermissions {
            module: Modules::Mint,
            permissions: vec![Permissions::Ownership],
        };
        let _ = app
            .execute_contract(
                Addr::unchecked(ADMIN),
                permission_module_addr,
                &msg,
                &vec![],
            )
            .unwrap();

        let permission_msg = to_binary(&vec![PermissionCheckMsg {
            permission_type: Permissions::Ownership,
            data: to_binary(&vec![
                OwnershipMsg {
                    bundle_id: 1,
                    token_id: 1,
                    owner: USER.to_string(),
                },
                OwnershipMsg {
                    bundle_id: 1,
                    token_id: 2,
                    owner: USER.to_string(),
                },
            ])
            .unwrap(),
        }])
        .unwrap();
        let bundle_ids = vec![2];
        let msg = MintExecuteMsg::PermissionMint {
            permission_msg,
            bundle_ids,
            metadata_ids: None,
        };
        let _ = app
            .execute_contract(
                Addr::unchecked(USER),
                mint_module_addr.clone(),
                &msg,
                &vec![],
            )
            .unwrap();

        let bundle_2_addr = query_bundle_address(&app.wrap(), &mint_module_addr, &2).unwrap();

        let msg = TokenQueryMsg::OwnerOf {
            token_id: "1".to_string(),
            include_expired: None,
        };
        let res: OwnerOfResponse = app.wrap().query_wasm_smart(bundle_2_addr, &msg).unwrap();
        assert_eq!(res.owner, USER);
    }
}
