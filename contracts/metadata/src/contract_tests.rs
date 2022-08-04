#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use crate::{
        msg::{ExecuteMsg, InstantiateMsg, MetadataResponse, QueryMsg},
        state::{Metadata, Trait},
    };

    pub fn metadata_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
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
        let metadata_code_id = app.store_code(metadata_contract());

        let msg = InstantiateMsg {
            admin: ADMIN.to_string(),
        };
        let metadata_contract_addr = app
            .instantiate_contract(
                metadata_code_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        metadata_contract_addr
    }

    fn setup_metadata(app: &mut App, metadata_contract_addr: Addr) {
        let attributes = vec![
            Trait {
                trait_type: "type_1".to_string(),
                value: "10".to_string(),
            },
            Trait {
                trait_type: "type_2".to_string(),
                value: "60".to_string(),
            },
            Trait {
                trait_type: "type_3".to_string(),
                value: "Banana".to_string(),
            },
        ];
        let metadata = Metadata {
            image: None,
            external_url: None,
            description: None,
            animation_url: None,
            youtube_url: None,
        };
        let msg = ExecuteMsg::AddMetadata {
            token_id: "token_id".to_string(),
            metadata: metadata.clone(),
            attributes: attributes.clone(),
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

    #[test]
    fn test_add_metadata() {
        let mut app = mock_app();
        let metadata_contract_addr = proper_instantiate(&mut app);

        let attributes = vec![
            Trait {
                trait_type: "type_1".to_string(),
                value: "10".to_string(),
            },
            Trait {
                trait_type: "type_2".to_string(),
                value: "60".to_string(),
            },
            Trait {
                trait_type: "type_3".to_string(),
                value: "Banana".to_string(),
            },
        ];
        let metadata = Metadata {
            image: None,
            external_url: None,
            description: None,
            animation_url: None,
            youtube_url: None,
        };
        let msg = ExecuteMsg::AddMetadata {
            token_id: "token_id".to_string(),
            metadata: metadata.clone(),
            attributes: attributes.clone(),
        };
        let _ = app
            .execute_contract(
                Addr::unchecked(ADMIN),
                metadata_contract_addr.clone(),
                &msg,
                &vec![],
            )
            .unwrap();

        let msg = QueryMsg::Metadata {
            token_id: "token_id".to_string(),
        };
        let res: MetadataResponse = app
            .wrap()
            .query_wasm_smart(metadata_contract_addr, &msg)
            .unwrap();
        assert_eq!(res.metadata, metadata);
        assert_eq!(res.attributes, attributes);
    }

    #[test]
    fn test_update_metadata() {
        let mut app = mock_app();
        let metadata_contract_addr = proper_instantiate(&mut app);

        setup_metadata(&mut app, metadata_contract_addr.clone());

        let metadata = Metadata {
            image: Some("image".to_string()),
            external_url: Some("external".to_string()),
            description: Some("description".to_string()),
            animation_url: Some("animation".to_string()),
            youtube_url: Some("youtube".to_string()),
        };
        let msg = ExecuteMsg::UpdateMetadata {
            token_id: "token_id".to_string(),
            metadata: metadata.clone(),
        };
        let _ = app
            .execute_contract(
                Addr::unchecked(ADMIN),
                metadata_contract_addr.clone(),
                &msg,
                &vec![],
            )
            .unwrap();

        let msg = QueryMsg::Metadata {
            token_id: "token_id".to_string(),
        };
        let res: MetadataResponse = app
            .wrap()
            .query_wasm_smart(metadata_contract_addr, &msg)
            .unwrap();
        assert_eq!(res.metadata, metadata);
    }

    #[test]
    fn test_add_attribute() {
        let mut app = mock_app();
        let metadata_contract_addr = proper_instantiate(&mut app);

        setup_metadata(&mut app, metadata_contract_addr.clone());

        let attribute = Trait {
            trait_type: "type_4".to_string(),
            value: "Cucumber".to_string(),
        };
        let msg = ExecuteMsg::AddAttribute {
            token_id: "token_id".to_string(),
            attribute,
        };
        let _ = app
            .execute_contract(
                Addr::unchecked(ADMIN),
                metadata_contract_addr.clone(),
                &msg,
                &vec![],
            )
            .unwrap();

        let msg = QueryMsg::Metadata {
            token_id: "token_id".to_string(),
        };
        let res: MetadataResponse = app
            .wrap()
            .query_wasm_smart(metadata_contract_addr, &msg)
            .unwrap();
        assert_eq!(
            res.attributes,
            vec![
                Trait {
                    trait_type: "type_1".to_string(),
                    value: "10".to_string(),
                },
                Trait {
                    trait_type: "type_2".to_string(),
                    value: "60".to_string(),
                },
                Trait {
                    trait_type: "type_3".to_string(),
                    value: "Banana".to_string(),
                },
                Trait {
                    trait_type: "type_4".to_string(),
                    value: "Cucumber".to_string(),
                }
            ]
        );
    }

    #[test]
    fn test_update_attribute() {
        let mut app = mock_app();
        let metadata_contract_addr = proper_instantiate(&mut app);

        setup_metadata(&mut app, metadata_contract_addr.clone());

        let attribute = Trait {
            trait_type: "type_2".to_string(),
            value: "Elephant".to_string(),
        };
        let msg = ExecuteMsg::UpdateAttribute {
            token_id: "token_id".to_string(),
            attribute,
        };
        let _ = app
            .execute_contract(
                Addr::unchecked(ADMIN),
                metadata_contract_addr.clone(),
                &msg,
                &vec![],
            )
            .unwrap();

        let msg = QueryMsg::Metadata {
            token_id: "token_id".to_string(),
        };
        let res: MetadataResponse = app
            .wrap()
            .query_wasm_smart(metadata_contract_addr, &msg)
            .unwrap();
        assert_eq!(
            res.attributes,
            vec![
                Trait {
                    trait_type: "type_1".to_string(),
                    value: "10".to_string(),
                },
                Trait {
                    trait_type: "type_2".to_string(),
                    value: "Elephant".to_string(),
                },
                Trait {
                    trait_type: "type_3".to_string(),
                    value: "Banana".to_string(),
                },
            ]
        );
    }

    #[test]
    fn test_remove_attribute() {
        let mut app = mock_app();
        let metadata_contract_addr = proper_instantiate(&mut app);

        setup_metadata(&mut app, metadata_contract_addr.clone());

        let msg = ExecuteMsg::RemoveAttribute {
            token_id: "token_id".to_string(),
            trait_type: "type_2".to_string(),
        };
        let _ = app
            .execute_contract(
                Addr::unchecked(ADMIN),
                metadata_contract_addr.clone(),
                &msg,
                &vec![],
            )
            .unwrap();

        let msg = QueryMsg::Metadata {
            token_id: "token_id".to_string(),
        };
        let res: MetadataResponse = app
            .wrap()
            .query_wasm_smart(metadata_contract_addr, &msg)
            .unwrap();
        assert_eq!(
            res.attributes,
            vec![
                Trait {
                    trait_type: "type_1".to_string(),
                    value: "10".to_string(),
                },
                Trait {
                    trait_type: "type_3".to_string(),
                    value: "Banana".to_string(),
                },
            ]
        );
    }
}
