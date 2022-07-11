#[cfg(test)]
mod tests {
    use crate::msg::InstantiateMsg;
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn controller_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply(crate::contract::reply);
        Box::new(contract)
    }

    pub fn mint_module_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            mint_module::contract::execute,
            mint_module::contract::instantiate,
            mint_module::contract::query,
        )
        .with_reply(mint_module::contract::reply);
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

    fn setup_mint_module() {}

    mod modules {
        use super::*;

        mod mint_module {
            use super::*;

            use crate::msg::{ExecuteMsg, ModuleAddrResponse, QueryMsg};

            #[test]
            fn test_init_module() {
                let mut app = mock_app();
                let controller_contract_addr = proper_instantiate(&mut app);
                let mint_module_code_id = app.store_code(mint_module_contract());

                let msg = ExecuteMsg::InitMintModule {
                    code_id: mint_module_code_id,
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        controller_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = QueryMsg::MintModuleAddr {};
                let res: ModuleAddrResponse = app
                    .wrap()
                    .query_wasm_smart(controller_contract_addr, &msg)
                    .unwrap();
                assert_eq!(res.addr, "contract1")
            }
        }
    }

    // mod test_collection {
    //     use super::*;

    //     use crate::{
    //         msg::{ExecuteMsg, GetCollectionResponse, QueryMsg},
    //         ContractError,
    //     };

    //     use mint::{
    //         msg::{InstantiateMsg as MintInstantiateMsg, QueryMsg as MintQueryMsg},
    //         state::Config,
    //     };

    //     #[test]
    //     fn test_happy_path() {
    //         let (mut app, controller_contract_addr) = proper_instantiate();
    //         let token_code_id = app.store_code(token_contract());

    //         let collection_info = CollectionInfo {
    //             name: "Test Collection".to_string(),
    //             description: "Test Collection".to_string(),
    //             image: "https://image.com".to_string(),
    //             external_link: None,
    //         };
    //         let instantiate_msg = MintInstantiateMsg {
    //             symbol: "TEST".to_string(),
    //             token_code_id,
    //             collection_info: collection_info.clone(),
    //             per_address_limit: None,
    //             whitelist: None,
    //             start_time: None,
    //         };
    //         let msg = ExecuteMsg::AddCollection { instantiate_msg };
    //         let _ = app
    //             .execute_contract(
    //                 Addr::unchecked(ADMIN),
    //                 controller_contract_addr.clone(),
    //                 &msg,
    //                 &vec![],
    //             )
    //             .unwrap();

    //         let msg = QueryMsg::GetCollection { collection_id: 1 };
    //         let response: GetCollectionResponse = app
    //             .wrap()
    //             .query_wasm_smart(controller_contract_addr, &msg)
    //             .unwrap();
    //         let collection_address = response.address;

    //         let msg = MintQueryMsg::GetCollectionInfo {};
    //         let response: CollectionInfo = app
    //             .wrap()
    //             .query_wasm_smart(collection_address, &msg)
    //             .unwrap();
    //         assert_eq!(response, collection_info);
    //     }

    //     #[test]
    //     fn test_mint_lock_happy_path() {
    //         let (mut app, controller_contract_addr) = proper_instantiate();
    //         let token_code_id = app.store_code(token_contract());

    //         let collection_info = CollectionInfo {
    //             name: "Test Collection".to_string(),
    //             description: "Test Collection".to_string(),
    //             image: "https://image.com".to_string(),
    //             external_link: None,
    //         };
    //         let instantiate_msg = MintInstantiateMsg {
    //             symbol: "TEST".to_string(),
    //             token_code_id,
    //             collection_info: collection_info.clone(),
    //             per_address_limit: None,
    //             whitelist: None,
    //             start_time: None,
    //         };
    //         let msg = ExecuteMsg::AddCollection { instantiate_msg };
    //         let _ = app
    //             .execute_contract(
    //                 Addr::unchecked(ADMIN),
    //                 controller_contract_addr.clone(),
    //                 &msg,
    //                 &vec![],
    //             )
    //             .unwrap();

    //         let err = app
    //             .execute_contract(
    //                 Addr::unchecked(USER),
    //                 controller_contract_addr.clone(),
    //                 &ExecuteMsg::UpdateMintLock {
    //                     collection_id: 1,
    //                     lock: true,
    //                 },
    //                 &vec![],
    //             )
    //             .unwrap_err();
    //         assert_eq!(
    //             err.source().unwrap().to_string(),
    //             ContractError::Unauthorized {}.to_string()
    //         );

    //         let _ = app.execute_contract(
    //             Addr::unchecked(ADMIN),
    //             controller_contract_addr.clone(),
    //             &ExecuteMsg::UpdateMintLock {
    //                 collection_id: 1,
    //                 lock: true,
    //             },
    //             &vec![],
    //         );

    //         // TODO: Use mint message here to test this
    //         let msg = QueryMsg::GetCollection { collection_id: 1 };
    //         let response: GetCollectionResponse = app
    //             .wrap()
    //             .query_wasm_smart(controller_contract_addr, &msg)
    //             .unwrap();
    //         let collection_address = response.address;

    //         let msg = MintQueryMsg::GetConfig {};
    //         let response: Config = app
    //             .wrap()
    //             .query_wasm_smart(collection_address, &msg)
    //             .unwrap();
    //         assert_eq!(response.mint_lock, true);
    //     }
    // }

    // mod test_code_id {
    //     use super::*;

    //     use crate::{
    //         msg::{ExecuteMsg, QueryMsg},
    //         state::Config,
    //         ContractError,
    //     };

    //     #[test]
    //     fn test_happy_path() {
    //         let (mut app, controller_contract_addr) = proper_instantiate();

    //         let msg = ExecuteMsg::UpdateMintCodeId { code_id: 100 };
    //         let _ = app.execute_contract(
    //             Addr::unchecked(ADMIN),
    //             controller_contract_addr.clone(),
    //             &msg,
    //             &vec![],
    //         );

    //         let msg = QueryMsg::GetConfig {};
    //         let response: Config = app
    //             .wrap()
    //             .query_wasm_smart(controller_contract_addr, &msg)
    //             .unwrap();
    //         assert_eq!(response.mint_code_id, 100);
    //     }

    //     #[test]
    //     fn test_unhappy_path() {
    //         let (mut app, controller_contract_addr) = proper_instantiate();

    //         let msg = ExecuteMsg::UpdateMintCodeId { code_id: 0 };
    //         let err = app
    //             .execute_contract(
    //                 Addr::unchecked(USER),
    //                 controller_contract_addr.clone(),
    //                 &msg,
    //                 &vec![],
    //             )
    //             .unwrap_err();
    //         assert_eq!(
    //             err.source().unwrap().to_string(),
    //             ContractError::Unauthorized {}.to_string()
    //         );

    //         let msg = ExecuteMsg::UpdateMintCodeId { code_id: 0 };
    //         let err = app
    //             .execute_contract(
    //                 Addr::unchecked(ADMIN),
    //                 controller_contract_addr.clone(),
    //                 &msg,
    //                 &vec![],
    //             )
    //             .unwrap_err();
    //         assert_eq!(
    //             err.source().unwrap().to_string(),
    //             ContractError::InvalidCodeId {}.to_string()
    //         );
    //     }
    // }
}
