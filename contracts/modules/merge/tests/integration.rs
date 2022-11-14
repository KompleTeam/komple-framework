use cosmwasm_std::{to_binary, Addr, Coin, Empty, Uint128};
use cw721::OwnerOfResponse;
use cw721_base::msg::{ExecuteMsg as Cw721ExecuteMsg, QueryMsg as Cw721QueryMsg};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_framework_hub_module::{
    msg::{ExecuteMsg as HubExecuteMsg, InstantiateMsg as HubInstantiateMsg},
    state::HubInfo,
};
use komple_framework_merge_module::msg::{
    ExecuteMsg as MergeModuleExecuteMsg, MergeBurnMsg, MergeMsg,
};
use komple_framework_merge_module::ContractError as MergeContractError;
use komple_metadata_module::msg::InstantiateMsg as MetadataModuleInstantiateMsg;
use komple_mint_module::{
    msg::{CollectionFundInfo, ExecuteMsg as MintModuleExecuteMsg},
    state::CollectionInfo,
};
use komple_permission_module::msg::ExecuteMsg as PermissionModuleExecuteMsg;
use komple_token_module::msg::{
    ExecuteMsg as TokenModuleExecuteMsg, MetadataInfo, QueryMsg as TokenModuleQueryMsg, TokenInfo,
};
use komple_token_module::state::CollectionConfig;
use komple_types::modules::metadata::Metadata as MetadataType;
use komple_types::modules::mint::Collections;
use komple_types::modules::permission::Permissions;
use komple_types::modules::Modules;
use komple_types::shared::RegisterMsg;
use komple_utils::storage::StorageHelper;

pub const USER: &str = "juno..user";
pub const RANDOM: &str = "juno..random";
pub const ADMIN: &str = "juno..admin";
pub const RANDOM_2: &str = "juno..random2";
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
        komple_mint_module::contract::execute,
        komple_mint_module::contract::instantiate,
        komple_mint_module::contract::query,
    )
    .with_reply(komple_mint_module::contract::reply);
    Box::new(contract)
}

pub fn permission_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_permission_module::contract::execute,
        komple_permission_module::contract::instantiate,
        komple_permission_module::contract::query,
    )
    .with_reply(komple_permission_module::contract::reply);
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

pub fn merge_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_framework_merge_module::contract::execute,
        komple_framework_merge_module::contract::instantiate,
        komple_framework_merge_module::contract::query,
    );
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

pub fn ownership_permission_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_ownership_permission_module::contract::execute,
        komple_ownership_permission_module::contract::instantiate,
        komple_ownership_permission_module::contract::query,
    );
    Box::new(contract)
}

pub fn link_permission_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_link_permission_module::contract::execute,
        komple_link_permission_module::contract::instantiate,
        komple_link_permission_module::contract::query,
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
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(RANDOM),
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
                &Addr::unchecked(RANDOM_2),
                vec![Coin {
                    denom: TEST_DENOM.to_string(),
                    amount: Uint128::new(1_000_000),
                }],
            )
            .unwrap();
    })
}

pub fn proper_instantiate(app: &mut App) -> Addr {
    let hub_code_id = app.store_code(hub_module());

    let msg = HubInstantiateMsg {
        hub_info: HubInfo {
            name: "Test Hub".to_string(),
            description: "Test Hub".to_string(),
            image: "https://image.com".to_string(),
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
        &[Coin {
            amount: Uint128::new(1_000_000),
            denom: NATIVE_DENOM.to_string(),
        }],
        "test",
        None,
    )
    .unwrap()
}

pub fn setup_mint_module(app: &mut App, hub_addr: Addr) {
    let mint_module_code_id = app.store_code(mint_module());

    let instantiate_msg = to_binary(&RegisterMsg {
        admin: ADMIN.to_string(),
        data: None,
    })
    .unwrap();
    let msg = HubExecuteMsg::RegisterModule {
        module: Modules::Mint.to_string(),
        msg: Some(instantiate_msg),
        code_id: mint_module_code_id,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), hub_addr, &msg, &[])
        .unwrap();
}

pub fn setup_merge_module(app: &mut App, hub_addr: Addr) {
    let merge_module_code_id = app.store_code(merge_module());

    let instantiate_msg = to_binary(&RegisterMsg {
        admin: ADMIN.to_string(),
        data: None,
    })
    .unwrap();
    let msg = HubExecuteMsg::RegisterModule {
        module: Modules::Merge.to_string(),
        msg: Some(instantiate_msg),
        code_id: merge_module_code_id,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), hub_addr, &msg, &[])
        .unwrap();
}

pub fn setup_permission_module(app: &mut App, hub_addr: Addr) {
    let permission_module_code_id = app.store_code(permission_module());

    let instantiate_msg = to_binary(&RegisterMsg {
        admin: ADMIN.to_string(),
        data: None,
    })
    .unwrap();
    let msg = HubExecuteMsg::RegisterModule {
        module: Modules::Permission.to_string(),
        msg: Some(instantiate_msg),
        code_id: permission_module_code_id,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), hub_addr, &msg, &[])
        .unwrap();
}

pub fn setup_all_modules(app: &mut App, hub_addr: Addr) {
    setup_mint_module(app, hub_addr.clone());
    setup_merge_module(app, hub_addr.clone());
    setup_permission_module(app, hub_addr.clone());
}

pub fn create_collection(
    app: &mut App,
    mint_module_addr: Addr,
    token_module_code_id: u64,
    linked_collections: Option<Vec<u32>>,
) {
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
        instantiate_msg: MetadataModuleInstantiateMsg {
            metadata_type: MetadataType::Standard,
        },
        code_id: metadata_code_id,
    };
    let fund_info = CollectionFundInfo {
        is_native: true,
        denom: NATIVE_DENOM.to_string(),
        cw20_address: None,
    };
    let msg = MintModuleExecuteMsg::CreateCollection {
        code_id: token_module_code_id,
        collection_config,
        collection_info,
        metadata_info,
        token_info,
        fund_info,
        linked_collections,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), mint_module_addr, &msg, &[])
        .unwrap();
}

pub fn link_collections(
    app: &mut App,
    mint_module_addr: Addr,
    collection_id: u32,
    linked_collections: Vec<u32>,
) {
    let msg = MintModuleExecuteMsg::UpdateLinkedCollections {
        collection_id,
        linked_collections,
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), mint_module_addr, &msg, &[])
        .unwrap();
}

pub fn mint_token(app: &mut App, mint_module_addr: Addr, collection_id: u32, sender: &str) {
    let msg = MintModuleExecuteMsg::Mint {
        collection_id,
        metadata_id: None,
    };
    let _ = app
        .execute_contract(Addr::unchecked(sender), mint_module_addr, &msg, &[])
        .unwrap();
}

pub fn give_approval_to_module(
    app: &mut App,
    token_module_addr: Addr,
    owner: &str,
    operator_addr: &Addr,
) {
    let msg: Cw721ExecuteMsg<Empty, TokenModuleExecuteMsg> = Cw721ExecuteMsg::ApproveAll {
        operator: operator_addr.to_string(),
        expires: None,
    };
    let _ = app
        .execute_contract(Addr::unchecked(owner), token_module_addr, &msg, &[])
        .unwrap();
}

pub fn setup_mint_module_operators(app: &mut App, mint_module_addr: Addr, addrs: Vec<String>) {
    let msg = MintModuleExecuteMsg::UpdateOperators { addrs };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), mint_module_addr, &msg, &[])
        .unwrap();
}

pub fn setup_module_permissions(
    app: &mut App,
    permission_module_addr: &Addr,
    module: String,
    permissions: Vec<String>,
) {
    let msg = PermissionModuleExecuteMsg::UpdateModulePermissions {
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

mod initialization {
    use super::*;

    use cosmwasm_std::to_binary;
    use komple_types::modules::Modules;

    use komple_framework_hub_module::ContractError;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let hub_addr = proper_instantiate(&mut app);
        let merge_module_code_id = app.store_code(merge_module());

        let instantiate_msg = to_binary(&RegisterMsg {
            admin: ADMIN.to_string(),
            data: None,
        })
        .unwrap();
        let msg = HubExecuteMsg::RegisterModule {
            module: Modules::Merge.to_string(),
            msg: Some(instantiate_msg),
            code_id: merge_module_code_id,
        };
        let _ = app.execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &[]);

        let res =
            StorageHelper::query_module_address(&app.wrap(), &hub_addr, Modules::Merge.to_string())
                .unwrap();
        assert_eq!(res, "contract1")
    }

    #[test]
    fn test_invalid_sender() {
        let mut app = mock_app();
        let hub_addr = proper_instantiate(&mut app);
        let merge_module_code_id = app.store_code(merge_module());

        let instantiate_msg = to_binary(&RegisterMsg {
            admin: ADMIN.to_string(),
            data: None,
        })
        .unwrap();
        let msg = HubExecuteMsg::RegisterModule {
            module: Modules::Merge.to_string(),
            msg: Some(instantiate_msg),
            code_id: merge_module_code_id,
        };
        let err = app
            .execute_contract(Addr::unchecked(USER), hub_addr, &msg, &[])
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            ContractError::Unauthorized {}.to_string()
        )
    }
}

mod normal_merge {
    use super::*;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let hub_addr = proper_instantiate(&mut app);

        setup_all_modules(&mut app, hub_addr.clone());

        let mint_module_addr =
            StorageHelper::query_module_address(&app.wrap(), &hub_addr, Modules::Mint.to_string())
                .unwrap();
        let merge_module_addr =
            StorageHelper::query_module_address(&app.wrap(), &hub_addr, Modules::Merge.to_string())
                .unwrap();

        let token_module_code_id = app.store_code(token_module());
        create_collection(
            &mut app,
            mint_module_addr.clone(),
            token_module_code_id,
            None,
        );
        create_collection(
            &mut app,
            mint_module_addr.clone(),
            token_module_code_id,
            None,
        );
        create_collection(
            &mut app,
            mint_module_addr.clone(),
            token_module_code_id,
            None,
        );

        link_collections(&mut app, mint_module_addr.clone(), 2, vec![3]);

        let collection_1_addr =
            StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();
        let collection_3_addr =
            StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &3).unwrap();

        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 3, USER);

        setup_mint_module_operators(
            &mut app,
            mint_module_addr.clone(),
            vec![merge_module_addr.to_string()],
        );

        give_approval_to_module(
            &mut app,
            collection_1_addr.clone(),
            USER,
            &merge_module_addr,
        );
        give_approval_to_module(&mut app, collection_3_addr, USER, &merge_module_addr);

        let merge_msg = MergeMsg {
            recipient: USER.to_string(),
            mint_id: 2,
            burn_ids: vec![
                MergeBurnMsg {
                    collection_id: 1,
                    token_id: 1,
                },
                MergeBurnMsg {
                    collection_id: 1,
                    token_id: 3,
                },
                MergeBurnMsg {
                    collection_id: 3,
                    token_id: 1,
                },
            ],
            metadata_id: None,
        };
        let msg = MergeModuleExecuteMsg::Merge { msg: merge_msg };
        let _ = app
            .execute_contract(Addr::unchecked(ADMIN), merge_module_addr, &msg, &[])
            .unwrap();

        let msg: Cw721QueryMsg<TokenModuleQueryMsg> = Cw721QueryMsg::OwnerOf {
            token_id: "1".to_string(),
            include_expired: None,
        };
        let res: Result<OwnerOfResponse, cosmwasm_std::StdError> =
            app.wrap().query_wasm_smart(collection_1_addr.clone(), &msg);
        assert!(res.is_err());

        let msg: Cw721QueryMsg<TokenModuleQueryMsg> = Cw721QueryMsg::OwnerOf {
            token_id: "3".to_string(),
            include_expired: None,
        };
        let res: Result<OwnerOfResponse, cosmwasm_std::StdError> =
            app.wrap().query_wasm_smart(collection_1_addr, &msg);
        assert!(res.is_err());

        let collection_2_addr =
            StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &2).unwrap();

        let msg: Cw721QueryMsg<TokenModuleQueryMsg> = Cw721QueryMsg::OwnerOf {
            token_id: "1".to_string(),
            include_expired: None,
        };
        let res: OwnerOfResponse = app
            .wrap()
            .query_wasm_smart(collection_2_addr, &msg)
            .unwrap();
        assert_eq!(res.owner, USER);
    }

    #[test]
    fn test_unhappy_path() {
        let mut app = mock_app();
        let hub_addr = proper_instantiate(&mut app);

        setup_all_modules(&mut app, hub_addr.clone());

        let mint_module_addr =
            StorageHelper::query_module_address(&app.wrap(), &hub_addr, Modules::Mint.to_string())
                .unwrap();
        let merge_module_addr =
            StorageHelper::query_module_address(&app.wrap(), &hub_addr, Modules::Merge.to_string())
                .unwrap();

        let token_module_code_id = app.store_code(token_module());
        create_collection(
            &mut app,
            mint_module_addr.clone(),
            token_module_code_id,
            None,
        );
        create_collection(
            &mut app,
            mint_module_addr.clone(),
            token_module_code_id,
            None,
        );
        create_collection(
            &mut app,
            mint_module_addr.clone(),
            token_module_code_id,
            Some(vec![2]),
        );

        mint_token(&mut app, mint_module_addr.clone(), 1, USER);

        let merge_msg = MergeMsg {
            recipient: USER.to_string(),
            mint_id: 2,
            burn_ids: vec![],
            metadata_id: None,
        };
        let msg = MergeModuleExecuteMsg::Merge { msg: merge_msg };
        let err = app
            .execute_contract(Addr::unchecked(ADMIN), merge_module_addr.clone(), &msg, &[])
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            MergeContractError::BurnNotFound {}.to_string()
        );

        let merge_msg = MergeMsg {
            recipient: USER.to_string(),
            mint_id: 2,
            burn_ids: vec![MergeBurnMsg {
                collection_id: 1,
                token_id: 1,
            }],
            metadata_id: None,
        };
        let msg = MergeModuleExecuteMsg::Merge { msg: merge_msg };
        let err = app
            .execute_contract(Addr::unchecked(ADMIN), merge_module_addr.clone(), &msg, &[])
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().source().unwrap().to_string(),
            MergeContractError::Unauthorized {}.to_string()
        );

        setup_mint_module_operators(
            &mut app,
            mint_module_addr.clone(),
            vec![merge_module_addr.to_string()],
        );

        let err = app
            .execute_contract(Addr::unchecked(USER), merge_module_addr.clone(), &msg, &[])
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().source().unwrap().to_string(),
            MergeContractError::Unauthorized {}.to_string()
        );

        setup_mint_module_operators(&mut app, mint_module_addr.clone(), vec![]);
        let collection_1_addr =
            StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &1).unwrap();
        give_approval_to_module(&mut app, collection_1_addr, USER, &merge_module_addr);

        let err = app
            .execute_contract(Addr::unchecked(USER), merge_module_addr.clone(), &msg, &[])
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().source().unwrap().to_string(),
            MergeContractError::Unauthorized {}.to_string()
        );
    }
}

mod permission_merge {
    use super::*;

    mod ownership_permission {
        use komple_ownership_permission_module::msg::OwnershipMsg;
        use komple_permission_module::msg::PermissionCheckMsg;

        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let hub_addr = proper_instantiate(&mut app);

            setup_all_modules(&mut app, hub_addr.clone());

            let mint_module_addr = StorageHelper::query_module_address(
                &app.wrap(),
                &hub_addr,
                Modules::Mint.to_string(),
            )
            .unwrap();
            let merge_module_addr = StorageHelper::query_module_address(
                &app.wrap(),
                &hub_addr,
                Modules::Merge.to_string(),
            )
            .unwrap();
            let permission_module_addr = StorageHelper::query_module_address(
                &app.wrap(),
                &hub_addr,
                Modules::Permission.to_string(),
            )
            .unwrap();

            let token_module_code_id = app.store_code(token_module());
            create_collection(
                &mut app,
                mint_module_addr.clone(),
                token_module_code_id,
                None,
            );
            create_collection(
                &mut app,
                mint_module_addr.clone(),
                token_module_code_id,
                None,
            );
            create_collection(
                &mut app,
                mint_module_addr.clone(),
                token_module_code_id,
                None,
            );

            link_collections(&mut app, mint_module_addr.clone(), 2, vec![3]);

            mint_token(&mut app, mint_module_addr.clone(), 1, USER);
            mint_token(&mut app, mint_module_addr.clone(), 1, USER);
            mint_token(&mut app, mint_module_addr.clone(), 1, USER);
            mint_token(&mut app, mint_module_addr.clone(), 3, USER);

            setup_mint_module_operators(
                &mut app,
                mint_module_addr.clone(),
                vec![merge_module_addr.to_string()],
            );

            let collection_1_addr =
                StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &1)
                    .unwrap();
            give_approval_to_module(
                &mut app,
                collection_1_addr.clone(),
                USER,
                &merge_module_addr,
            );
            let collection_3_addr =
                StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &3)
                    .unwrap();
            give_approval_to_module(&mut app, collection_3_addr, USER, &merge_module_addr);

            let ownership_permission_code_id = app.store_code(ownership_permission_module());
            let msg = PermissionModuleExecuteMsg::RegisterPermission {
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

            setup_module_permissions(
                &mut app,
                &permission_module_addr,
                Modules::Merge.to_string(),
                vec![Permissions::Ownership.to_string()],
            );

            let permission_msg = to_binary(&vec![PermissionCheckMsg {
                permission_type: Permissions::Ownership.to_string(),
                data: to_binary(&vec![
                    OwnershipMsg {
                        collection_id: 1,
                        token_id: 1,
                        address: USER.to_string(),
                    },
                    OwnershipMsg {
                        collection_id: 1,
                        token_id: 2,
                        address: USER.to_string(),
                    },
                ])
                .unwrap(),
            }])
            .unwrap();
            let merge_msg = MergeMsg {
                recipient: USER.to_string(),
                mint_id: 2,
                burn_ids: vec![
                    MergeBurnMsg {
                        collection_id: 1,
                        token_id: 1,
                    },
                    MergeBurnMsg {
                        collection_id: 1,
                        token_id: 3,
                    },
                    MergeBurnMsg {
                        collection_id: 3,
                        token_id: 1,
                    },
                ],
                metadata_id: None,
            };
            let msg = MergeModuleExecuteMsg::PermissionMerge {
                permission_msg,
                merge_msg,
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), merge_module_addr, &msg, &[])
                .unwrap();

            let msg: Cw721QueryMsg<TokenModuleQueryMsg> = Cw721QueryMsg::OwnerOf {
                token_id: "1".to_string(),
                include_expired: None,
            };
            let res: Result<OwnerOfResponse, cosmwasm_std::StdError> =
                app.wrap().query_wasm_smart(collection_1_addr.clone(), &msg);
            assert!(res.is_err());

            let msg: Cw721QueryMsg<TokenModuleQueryMsg> = Cw721QueryMsg::OwnerOf {
                token_id: "3".to_string(),
                include_expired: None,
            };
            let res: Result<OwnerOfResponse, cosmwasm_std::StdError> =
                app.wrap().query_wasm_smart(collection_1_addr, &msg);
            assert!(res.is_err());

            let collection_2_addr =
                StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &2)
                    .unwrap();

            let msg: Cw721QueryMsg<TokenModuleQueryMsg> = Cw721QueryMsg::OwnerOf {
                token_id: "1".to_string(),
                include_expired: None,
            };
            let res: OwnerOfResponse = app
                .wrap()
                .query_wasm_smart(collection_2_addr, &msg)
                .unwrap();
            assert_eq!(res.owner, USER);
        }
    }

    mod link_permission {
        use komple_link_permission_module::{
            msg::LinkPermissionMsg, ContractError as LinkPermissionError,
        };
        use komple_permission_module::msg::PermissionCheckMsg;

        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let hub_addr = proper_instantiate(&mut app);

            setup_all_modules(&mut app, hub_addr.clone());

            let mint_module_addr = StorageHelper::query_module_address(
                &app.wrap(),
                &hub_addr,
                Modules::Mint.to_string(),
            )
            .unwrap();
            let merge_module_addr = StorageHelper::query_module_address(
                &app.wrap(),
                &hub_addr,
                Modules::Merge.to_string(),
            )
            .unwrap();
            let permission_module_addr = StorageHelper::query_module_address(
                &app.wrap(),
                &hub_addr,
                Modules::Permission.to_string(),
            )
            .unwrap();

            let token_module_code_id = app.store_code(token_module());
            create_collection(
                &mut app,
                mint_module_addr.clone(),
                token_module_code_id,
                None,
            );
            create_collection(
                &mut app,
                mint_module_addr.clone(),
                token_module_code_id,
                None,
            );
            create_collection(
                &mut app,
                mint_module_addr.clone(),
                token_module_code_id,
                Some(vec![1, 2]),
            );

            mint_token(&mut app, mint_module_addr.clone(), 1, USER);
            mint_token(&mut app, mint_module_addr.clone(), 2, USER);

            setup_mint_module_operators(
                &mut app,
                mint_module_addr.clone(),
                vec![merge_module_addr.to_string()],
            );

            let collection_1_addr =
                StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &1)
                    .unwrap();
            give_approval_to_module(
                &mut app,
                collection_1_addr.clone(),
                USER,
                &merge_module_addr,
            );
            let collection_2_addr =
                StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &2)
                    .unwrap();
            give_approval_to_module(
                &mut app,
                collection_2_addr.clone(),
                USER,
                &merge_module_addr,
            );

            let link_permission_code_id = app.store_code(link_permission_module());
            let msg = PermissionModuleExecuteMsg::RegisterPermission {
                permission: Permissions::Link.to_string(),
                msg: Some(
                    to_binary(&RegisterMsg {
                        admin: ADMIN.to_string(),
                        data: None,
                    })
                    .unwrap(),
                ),
                code_id: link_permission_code_id,
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    permission_module_addr.clone(),
                    &msg,
                    &[],
                )
                .unwrap();

            setup_module_permissions(
                &mut app,
                &permission_module_addr,
                Modules::Merge.to_string(),
                vec![Permissions::Link.to_string()],
            );

            let permission_msg = to_binary(&vec![PermissionCheckMsg {
                permission_type: Permissions::Link.to_string(),
                data: to_binary(&vec![LinkPermissionMsg {
                    collection_id: 3,
                    collection_ids: vec![1],
                }])
                .unwrap(),
            }])
            .unwrap();
            let merge_msg = MergeMsg {
                recipient: USER.to_string(),
                mint_id: 3,
                burn_ids: vec![
                    MergeBurnMsg {
                        collection_id: 1,
                        token_id: 1,
                    },
                    MergeBurnMsg {
                        collection_id: 2,
                        token_id: 1,
                    },
                ],
                metadata_id: None,
            };
            let msg = MergeModuleExecuteMsg::PermissionMerge {
                permission_msg,
                merge_msg: merge_msg.clone(),
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), merge_module_addr.clone(), &msg, &[])
                .unwrap_err();
            // Three errors because we have merge -> permission -> link permission
            assert_eq!(
                err.source()
                    .unwrap()
                    .source()
                    .unwrap()
                    .source()
                    .unwrap()
                    .to_string(),
                LinkPermissionError::LinkedCollectionNotFound {}.to_string()
            );

            let permission_msg = to_binary(&vec![PermissionCheckMsg {
                permission_type: Permissions::Link.to_string(),
                data: to_binary(&vec![LinkPermissionMsg {
                    collection_id: 3,
                    collection_ids: vec![1, 2],
                }])
                .unwrap(),
            }])
            .unwrap();
            let msg = MergeModuleExecuteMsg::PermissionMerge {
                permission_msg,
                merge_msg,
            };
            app.execute_contract(Addr::unchecked(ADMIN), merge_module_addr, &msg, &[])
                .unwrap();

            let msg: Cw721QueryMsg<TokenModuleQueryMsg> = Cw721QueryMsg::OwnerOf {
                token_id: "1".to_string(),
                include_expired: None,
            };
            let res: Result<OwnerOfResponse, cosmwasm_std::StdError> =
                app.wrap().query_wasm_smart(collection_1_addr.clone(), &msg);
            assert!(res.is_err());

            let msg: Cw721QueryMsg<TokenModuleQueryMsg> = Cw721QueryMsg::OwnerOf {
                token_id: "1".to_string(),
                include_expired: None,
            };
            let res: Result<OwnerOfResponse, cosmwasm_std::StdError> =
                app.wrap().query_wasm_smart(collection_2_addr, &msg);
            assert!(res.is_err());

            let collection_3_addr =
                StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &3)
                    .unwrap();

            let msg: Cw721QueryMsg<TokenModuleQueryMsg> = Cw721QueryMsg::OwnerOf {
                token_id: "1".to_string(),
                include_expired: None,
            };
            let res: OwnerOfResponse = app
                .wrap()
                .query_wasm_smart(collection_3_addr, &msg)
                .unwrap();
            assert_eq!(res.owner, USER);
        }
    }
}
