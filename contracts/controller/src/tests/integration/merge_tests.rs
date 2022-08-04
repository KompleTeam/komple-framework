#[cfg(test)]
mod tests {
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use cosmwasm_std::{Addr, Coin, Empty, Timestamp, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
    use mint_module::msg::{ExecuteMsg as MintExecuteMsg, QueryMsg as MintQueryMsg};
    use rift_types::{module::Modules, query::AddressResponse};
    use token_contract::{
        msg::{ExecuteMsg as TokenExecuteMsg, TokenInfo},
        state::CollectionInfo,
    };

    pub fn controller_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply(crate::contract::reply);
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

    const USER: &str = "juno1shfqtuup76mngspx29gcquykjvvlx9na4kymlm";
    const ADMIN: &str = "juno1qamfln8u5w8d3vlhp5t9mhmylfkgad4jz6t7cv";
    const NATIVE_DENOM: &str = "denom";

    fn mock_app() -> App {
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

    fn proper_instantiate(app: &mut App) -> Addr {
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

    fn setup_mint_module(app: &mut App, controller_addr: Addr) {
        let mint_module_code_id = app.store_code(mint_module());

        let msg = ExecuteMsg::InitMintModule {
            code_id: mint_module_code_id,
        };
        let _ = app
            .execute_contract(Addr::unchecked(ADMIN), controller_addr, &msg, &vec![])
            .unwrap();
    }

    fn setup_merge_module(app: &mut App, controller_addr: Addr) {
        let merge_module_code_id = app.store_code(merge_module());

        let msg = ExecuteMsg::InitMergeModule {
            code_id: merge_module_code_id,
        };
        let _ = app
            .execute_contract(Addr::unchecked(ADMIN), controller_addr, &msg, &vec![])
            .unwrap();
    }

    fn setup_permission_module(app: &mut App, controller_addr: Addr) {
        let permission_module_code_id = app.store_code(permission_module());

        let msg = ExecuteMsg::InitPermissionModule {
            code_id: permission_module_code_id,
        };
        let _ = app
            .execute_contract(Addr::unchecked(ADMIN), controller_addr, &msg, &vec![])
            .unwrap();
    }

    fn setup_all_modules(app: &mut App, controller_addr: Addr) {
        setup_mint_module(app, controller_addr.clone());
        setup_merge_module(app, controller_addr.clone());
        setup_permission_module(app, controller_addr.clone());
    }

    fn create_collection(
        app: &mut App,
        mint_module_addr: Addr,
        token_contract_code_id: u64,
        per_address_limit: Option<u32>,
        start_time: Option<Timestamp>,
        whitelist: Option<String>,
        royalty: Option<String>,
    ) {
        let collection_info = CollectionInfo {
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
            whitelist,
            royalty,
        };
        let _ = app
            .execute_contract(Addr::unchecked(ADMIN), mint_module_addr, &msg, &vec![])
            .unwrap();
    }

    fn mint_token(app: &mut App, mint_module_addr: Addr, collection_id: u32, sender: &str) {
        let msg = MintExecuteMsg::Mint { collection_id };
        let _ = app
            .execute_contract(Addr::unchecked(sender), mint_module_addr, &msg, &vec![])
            .unwrap();
    }

    fn setup_mint_module_whitelist(app: &mut App, mint_module_addr: Addr, addrs: Vec<String>) {
        let msg = MintExecuteMsg::UpdateWhitelistAddresses { addrs };
        let _ = app
            .execute_contract(Addr::unchecked(ADMIN), mint_module_addr, &msg, &vec![])
            .unwrap();
    }

    fn give_approval_to_module(
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

    fn get_modules_addresses(app: &mut App, controller_addr: &str) -> (Addr, Addr, Addr) {
        let mint_module_addr: Addr;
        let merge_module_addr: Addr;
        let permission_module_addr: Addr;

        let msg = QueryMsg::ModuleAddress(Modules::MintModule);
        let res = app.wrap().query_wasm_smart(controller_addr, &msg);
        let res: AddressResponse = res.unwrap();
        mint_module_addr = Addr::unchecked(res.address);

        let msg = QueryMsg::ModuleAddress(Modules::MergeModule);
        let res = app.wrap().query_wasm_smart(controller_addr, &msg);
        let res: AddressResponse = res.unwrap();
        merge_module_addr = Addr::unchecked(res.address);

        let msg = QueryMsg::ModuleAddress(Modules::PermissionModule);
        let res = app.wrap().query_wasm_smart(controller_addr, &msg);
        let res: AddressResponse = res.unwrap();
        permission_module_addr = Addr::unchecked(res.address);

        // println!("");
        // println!("mint_module_addr: {}", mint_module_addr);
        // println!("merge_module_addr: {}", merge_module_addr);
        // println!("permission_module_addr: {}", permission_module_addr);
        // println!("");

        (mint_module_addr, merge_module_addr, permission_module_addr)
    }

    fn get_collection_address(app: &mut App, mint_module_addr: &str, collection_id: u32) -> Addr {
        let msg = MintQueryMsg::CollectionAddress(collection_id);
        let res: AddressResponse = app.wrap().query_wasm_smart(mint_module_addr, &msg).unwrap();
        Addr::unchecked(res.address)
    }

    mod initialization {
        use super::*;

        use rift_types::{module::Modules, query::AddressResponse};

        use crate::{msg::QueryMsg, ContractError};

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let controller_addr = proper_instantiate(&mut app);
            let merge_module_code_id = app.store_code(merge_module());

            let msg = ExecuteMsg::InitMergeModule {
                code_id: merge_module_code_id,
            };
            let _ = app.execute_contract(
                Addr::unchecked(ADMIN),
                controller_addr.clone(),
                &msg,
                &vec![],
            );

            let msg = QueryMsg::ModuleAddress(Modules::MergeModule);
            let res: AddressResponse = app.wrap().query_wasm_smart(controller_addr, &msg).unwrap();
            assert_eq!(res.address, "contract1")
        }

        #[test]
        fn test_invalid_sender() {
            let mut app = mock_app();
            let controller_addr = proper_instantiate(&mut app);
            let merge_module_code_id = app.store_code(merge_module());

            let msg = ExecuteMsg::InitMergeModule {
                code_id: merge_module_code_id,
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(USER),
                    controller_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            )
        }
    }

    mod normal_merge {
        use crate::ContractError;

        use super::*;

        use cosmwasm_std::to_binary;
        use cw721::OwnerOfResponse;
        use merge_module::{
            msg::{ExecuteMsg as MergeExecuteMsg, MergeAction, MergeMsg},
            ContractError as MergeContractError,
        };
        use rift_types::collection::Collections;
        use token_contract::msg::QueryMsg as TokenQueryMsg;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let controller_addr = proper_instantiate(&mut app);

            setup_all_modules(&mut app, controller_addr.clone());

            let (mint_module_addr, merge_module_addr, _) =
                get_modules_addresses(&mut app, &controller_addr.to_string());

            let token_contract_code_id = app.store_code(token_contract());
            create_collection(
                &mut app,
                mint_module_addr.clone(),
                token_contract_code_id,
                None,
                None,
                None,
                None,
            );
            create_collection(
                &mut app,
                mint_module_addr.clone(),
                token_contract_code_id,
                None,
                None,
                None,
                None,
            );

            mint_token(&mut app, mint_module_addr.clone(), 1, USER);
            mint_token(&mut app, mint_module_addr.clone(), 1, USER);
            mint_token(&mut app, mint_module_addr.clone(), 1, USER);

            setup_mint_module_whitelist(
                &mut app,
                mint_module_addr.clone(),
                vec![merge_module_addr.to_string()],
            );

            let collection_1_addr =
                get_collection_address(&mut app, &mint_module_addr.to_string(), 1);
            give_approval_to_module(
                &mut app,
                collection_1_addr.clone(),
                USER,
                &merge_module_addr,
            );

            let merge_msg = vec![
                MergeMsg {
                    collection_id: 1,
                    token_id: Some(1),
                    collection_type: Collections::Normal,
                    action: MergeAction::Burn,
                },
                MergeMsg {
                    collection_id: 1,
                    token_id: Some(3),
                    collection_type: Collections::Normal,
                    action: MergeAction::Burn,
                },
                MergeMsg {
                    collection_id: 2,
                    token_id: None,
                    collection_type: Collections::Normal,
                    action: MergeAction::Mint,
                },
            ];
            let msg = MergeExecuteMsg::Merge {
                msg: to_binary(&merge_msg).unwrap(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(USER), merge_module_addr, &msg, &vec![])
                .unwrap();

            let msg = TokenQueryMsg::OwnerOf {
                token_id: "1".to_string(),
                include_expired: None,
            };
            let res: Result<OwnerOfResponse, cosmwasm_std::StdError> =
                app.wrap().query_wasm_smart(collection_1_addr.clone(), &msg);
            assert!(res.is_err());

            let msg = TokenQueryMsg::OwnerOf {
                token_id: "3".to_string(),
                include_expired: None,
            };
            let res: Result<OwnerOfResponse, cosmwasm_std::StdError> =
                app.wrap().query_wasm_smart(collection_1_addr.clone(), &msg);
            assert!(res.is_err());

            let collection_2_addr =
                get_collection_address(&mut app, &mint_module_addr.to_string(), 2);

            let msg = TokenQueryMsg::OwnerOf {
                token_id: "1".to_string(),
                include_expired: None,
            };
            let res: OwnerOfResponse = app
                .wrap()
                .query_wasm_smart(collection_2_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.owner, USER);
        }

        fn test_unhappy_path() {
            let mut app = mock_app();
            let controller_addr = proper_instantiate(&mut app);

            setup_all_modules(&mut app, controller_addr.clone());

            let (mint_module_addr, merge_module_addr, _) =
                get_modules_addresses(&mut app, &controller_addr.to_string());

            let merge_msg = vec![MergeMsg {
                collection_id: 2,
                token_id: None,
                collection_type: Collections::Passcard,
                action: MergeAction::Mint,
            }];
            let msg = MergeExecuteMsg::Merge {
                msg: to_binary(&merge_msg).unwrap(),
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(USER),
                    merge_module_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                MergeContractError::BurnNotFound {}.to_string()
            );

            let merge_msg = vec![
                MergeMsg {
                    collection_id: 1,
                    token_id: Some(1),
                    collection_type: Collections::Normal,
                    action: MergeAction::Burn,
                },
                MergeMsg {
                    collection_id: 2,
                    token_id: None,
                    collection_type: Collections::Passcard,
                    action: MergeAction::Mint,
                },
            ];
            let msg = MergeExecuteMsg::Merge {
                msg: to_binary(&merge_msg).unwrap(),
            };
            let err = app
                .execute_contract(Addr::unchecked(USER), merge_module_addr, &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                MergeContractError::InvalidPasscard {}.to_string()
            );
        }
    }
}
