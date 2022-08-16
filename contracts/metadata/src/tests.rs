use crate::msg::MetadataResponse;
use crate::ContractError;
use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{MetaInfo, Metadata, Trait},
};
use cosmwasm_std::{Addr, Coin, Empty, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_types::metadata::Metadata as MetadataType;
use komple_types::query::ResponseWrapper;

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

fn proper_instantiate(app: &mut App, metadata_type: MetadataType) -> Addr {
    let metadata_code_id = app.store_code(metadata_contract());

    let msg = InstantiateMsg {
        admin: ADMIN.to_string(),
        metadata_type,
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

fn setup_metadata(app: &mut App, metadata_contract_addr: Addr) -> (Vec<Trait>, MetaInfo) {
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
    let meta_info = MetaInfo {
        image: Some("https://example.com/image.png".to_string()),
        external_url: None,
        description: None,
        animation_url: None,
        youtube_url: None,
    };
    let msg = ExecuteMsg::AddMetadata {
        meta_info: meta_info.clone(),
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
    (attributes, meta_info)
}

mod initialization {
    use super::*;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let metadata_code_id = app.store_code(metadata_contract());

        let msg = InstantiateMsg {
            admin: ADMIN.to_string(),
            metadata_type: MetadataType::Static,
        };
        let _ = app
            .instantiate_contract(
                metadata_code_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();
    }
}

mod actions {
    use super::*;

    mod add_metadata {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::Static);

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
            let meta_info = MetaInfo {
                image: Some("https://example.com/image.png".to_string()),
                external_url: None,
                description: None,
                animation_url: None,
                youtube_url: None,
            };
            let msg = ExecuteMsg::AddMetadata {
                meta_info: meta_info.clone(),
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

            let msg = QueryMsg::RawMetadata { metadata_id: 1 };
            let res: ResponseWrapper<Metadata> = app
                .wrap()
                .query_wasm_smart(metadata_contract_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.attributes, attributes);
            assert_eq!(res.data.meta_info, meta_info);
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::OneToOne);

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
            let meta_info = MetaInfo {
                image: Some("https://example.com/image.png".to_string()),
                external_url: None,
                description: None,
                animation_url: None,
                youtube_url: None,
            };
            let msg = ExecuteMsg::AddMetadata {
                meta_info: meta_info.clone(),
                attributes: attributes.clone(),
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(USER),
                    metadata_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }
    }

    mod link_metadata {
        use super::*;

        mod one_to_one_metadata {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::OneToOne);

                let (attributes, meta_info) =
                    setup_metadata(&mut app, metadata_contract_addr.clone());

                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 1,
                    metadata_id: None,
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = QueryMsg::Metadata { token_id: 1 };
                let res: ResponseWrapper<MetadataResponse> = app
                    .wrap()
                    .query_wasm_smart(metadata_contract_addr.clone(), &msg)
                    .unwrap();
                assert_eq!(res.data.metadata.attributes, attributes);
                assert_eq!(res.data.metadata.meta_info, meta_info);
            }
        }

        mod static_metadata {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::Static);

                let (attributes, meta_info) =
                    setup_metadata(&mut app, metadata_contract_addr.clone());

                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 1,
                    metadata_id: Some(1),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = QueryMsg::Metadata { token_id: 1 };
                let res: ResponseWrapper<MetadataResponse> = app
                    .wrap()
                    .query_wasm_smart(metadata_contract_addr.clone(), &msg)
                    .unwrap();
                assert_eq!(res.data.metadata.attributes, attributes);
                assert_eq!(res.data.metadata.meta_info, meta_info);
            }

            #[test]
            fn test_missing_metadata() {
                let mut app = mock_app();
                let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::Static);

                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 1,
                    metadata_id: None,
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::MissingMetadata {}.to_string()
                )
            }
        }

        mod static_dynamic_metadata {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::Dynamic);

                let (attributes, meta_info) =
                    setup_metadata(&mut app, metadata_contract_addr.clone());

                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 1,
                    metadata_id: Some(1),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = QueryMsg::Metadata { token_id: 1 };
                let res: ResponseWrapper<MetadataResponse> = app
                    .wrap()
                    .query_wasm_smart(metadata_contract_addr.clone(), &msg)
                    .unwrap();
                assert_eq!(res.data.metadata.attributes, attributes);
                assert_eq!(res.data.metadata.meta_info, meta_info);
            }

            #[test]
            fn test_missing_metadata() {
                let mut app = mock_app();
                let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::Dynamic);

                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 1,
                    metadata_id: None,
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::MissingMetadata {}.to_string()
                )
            }
        }

        #[test]
        fn test_missing_metadata() {
            let mut app = mock_app();
            let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::OneToOne);

            let msg = ExecuteMsg::LinkMetadata {
                token_id: 1,
                metadata_id: None,
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MissingMetadata {}.to_string()
            );
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::OneToOne);

            let msg = ExecuteMsg::LinkMetadata {
                token_id: 1,
                metadata_id: None,
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(USER),
                    metadata_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }
    }

    mod update_meta_info {
        use super::*;

        mod one_to_one_and_static_metadata {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::OneToOne);
                let metadata_contract_addr_2 = proper_instantiate(&mut app, MetadataType::Static);

                setup_metadata(&mut app, metadata_contract_addr.clone());
                setup_metadata(&mut app, metadata_contract_addr_2.clone());

                let new_meta_info = MetaInfo {
                    image: Some("https://test".to_string()),
                    description: Some("test".to_string()),
                    external_url: Some("https://test".to_string()),
                    animation_url: Some("https://test".to_string()),
                    youtube_url: Some("https://test".to_string()),
                };

                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 1,
                    metadata_id: Some(1),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr_2.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = ExecuteMsg::UpdateMetaInfo {
                    token_id: 1,
                    meta_info: new_meta_info.clone(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr_2.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = QueryMsg::Metadata { token_id: 1 };
                let res: ResponseWrapper<MetadataResponse> = app
                    .wrap()
                    .query_wasm_smart(metadata_contract_addr.clone(), &msg)
                    .unwrap();
                assert_eq!(res.data.metadata.meta_info, new_meta_info);
                let res: ResponseWrapper<MetadataResponse> = app
                    .wrap()
                    .query_wasm_smart(metadata_contract_addr_2.clone(), &msg)
                    .unwrap();
                assert_eq!(res.data.metadata.meta_info, new_meta_info);
            }

            #[test]
            fn test_missing_metadata() {
                let mut app = mock_app();
                let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::OneToOne);
                let metadata_contract_addr_2 = proper_instantiate(&mut app, MetadataType::Static);

                let new_meta_info = MetaInfo {
                    image: Some("https://test".to_string()),
                    description: Some("test".to_string()),
                    external_url: Some("https://test".to_string()),
                    animation_url: Some("https://test".to_string()),
                    youtube_url: Some("https://test".to_string()),
                };

                let msg = ExecuteMsg::UpdateMetaInfo {
                    token_id: 1,
                    meta_info: new_meta_info.clone(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::MissingMetadata {}.to_string()
                );
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr_2.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::MissingMetadata {}.to_string()
                );
            }
        }

        mod dynamic_metadata {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::Dynamic);

                setup_metadata(&mut app, metadata_contract_addr.clone());

                let new_meta_info = MetaInfo {
                    image: Some("https://test".to_string()),
                    description: Some("test".to_string()),
                    external_url: Some("https://test".to_string()),
                    animation_url: Some("https://test".to_string()),
                    youtube_url: Some("https://test".to_string()),
                };

                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 1,
                    metadata_id: Some(1),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = ExecuteMsg::UpdateMetaInfo {
                    token_id: 1,
                    meta_info: new_meta_info.clone(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg = QueryMsg::Metadata { token_id: 1 };
                let res: ResponseWrapper<MetadataResponse> = app
                    .wrap()
                    .query_wasm_smart(metadata_contract_addr.clone(), &msg)
                    .unwrap();
                assert_eq!(res.data.metadata.meta_info, new_meta_info);
            }

            #[test]
            fn test_missing_metadata() {
                let mut app = mock_app();
                let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::Dynamic);

                let new_meta_info = MetaInfo {
                    image: Some("https://test".to_string()),
                    description: Some("test".to_string()),
                    external_url: Some("https://test".to_string()),
                    animation_url: Some("https://test".to_string()),
                    youtube_url: Some("https://test".to_string()),
                };

                let msg = ExecuteMsg::UpdateMetaInfo {
                    token_id: 1,
                    meta_info: new_meta_info.clone(),
                };
                let err = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::MissingMetadata {}.to_string()
                );
            }
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::OneToOne);

            let new_meta_info = MetaInfo {
                image: Some("https://test".to_string()),
                description: Some("test".to_string()),
                external_url: Some("https://test".to_string()),
                animation_url: Some("https://test".to_string()),
                youtube_url: Some("https://test".to_string()),
            };

            let msg = ExecuteMsg::UpdateMetaInfo {
                token_id: 1,
                meta_info: new_meta_info.clone(),
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(USER),
                    metadata_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }
    }

    mod add_attribute {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::OneToOne);
            let metadata_contract_addr_2 = proper_instantiate(&mut app, MetadataType::Static);
            let metadata_contract_addr_3 = proper_instantiate(&mut app, MetadataType::Dynamic);

            setup_metadata(&mut app, metadata_contract_addr.clone());
            setup_metadata(&mut app, metadata_contract_addr_2.clone());
            setup_metadata(&mut app, metadata_contract_addr_3.clone());

            let attribute = Trait {
                trait_type: "new_trait".to_string(),
                value: "some_value".to_string(),
            };

            let msg = ExecuteMsg::LinkMetadata {
                token_id: 1,
                metadata_id: Some(1),
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_2.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_3.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let msg = ExecuteMsg::AddAttribute {
                token_id: 1,
                attribute: attribute.clone(),
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_2.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_3.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let new_attributes = vec![
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
                attribute.clone(),
            ];

            let msg = QueryMsg::Metadata { token_id: 1 };
            let res: ResponseWrapper<MetadataResponse> = app
                .wrap()
                .query_wasm_smart(metadata_contract_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.metadata.attributes, new_attributes);
            let res: ResponseWrapper<MetadataResponse> = app
                .wrap()
                .query_wasm_smart(metadata_contract_addr_2.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.metadata.attributes, new_attributes);
            let res: ResponseWrapper<MetadataResponse> = app
                .wrap()
                .query_wasm_smart(metadata_contract_addr_3.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.metadata.attributes, new_attributes);
        }

        #[test]
        fn test_existing_attribute() {
            let mut app = mock_app();
            let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::OneToOne);
            let metadata_contract_addr_2 = proper_instantiate(&mut app, MetadataType::Static);
            let metadata_contract_addr_3 = proper_instantiate(&mut app, MetadataType::Dynamic);

            setup_metadata(&mut app, metadata_contract_addr.clone());
            setup_metadata(&mut app, metadata_contract_addr_2.clone());
            setup_metadata(&mut app, metadata_contract_addr_3.clone());

            let msg = ExecuteMsg::LinkMetadata {
                token_id: 1,
                metadata_id: Some(1),
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_2.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_3.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let attribute = Trait {
                trait_type: "type_1".to_string(),
                value: "some_value".to_string(),
            };
            let msg = ExecuteMsg::AddAttribute {
                token_id: 1,
                attribute: attribute.clone(),
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AttributeAlreadyExists {}.to_string()
            );
            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_2.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AttributeAlreadyExists {}.to_string()
            );
            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_3.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AttributeAlreadyExists {}.to_string()
            );
        }

        #[test]
        fn test_missing_metadata() {
            let mut app = mock_app();
            let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::OneToOne);
            let metadata_contract_addr_2 = proper_instantiate(&mut app, MetadataType::Static);
            let metadata_contract_addr_3 = proper_instantiate(&mut app, MetadataType::Dynamic);

            let attribute = Trait {
                trait_type: "new_trait".to_string(),
                value: "some_value".to_string(),
            };
            let msg = ExecuteMsg::AddAttribute {
                token_id: 1,
                attribute: attribute.clone(),
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MissingMetadata {}.to_string()
            );
            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_2.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MissingMetadata {}.to_string()
            );
            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_3.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MissingMetadata {}.to_string()
            );
        }
    }

    mod update_attribute {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::OneToOne);
            let metadata_contract_addr_2 = proper_instantiate(&mut app, MetadataType::Static);
            let metadata_contract_addr_3 = proper_instantiate(&mut app, MetadataType::Dynamic);

            setup_metadata(&mut app, metadata_contract_addr.clone());
            setup_metadata(&mut app, metadata_contract_addr_2.clone());
            setup_metadata(&mut app, metadata_contract_addr_3.clone());

            let attribute = Trait {
                trait_type: "type_2".to_string(),
                value: "some_value".to_string(),
            };

            let msg = ExecuteMsg::LinkMetadata {
                token_id: 1,
                metadata_id: Some(1),
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_2.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_3.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let msg = ExecuteMsg::UpdateAttribute {
                token_id: 1,
                attribute: attribute.clone(),
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_2.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_3.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let new_attributes = vec![
                Trait {
                    trait_type: "type_1".to_string(),
                    value: "10".to_string(),
                },
                Trait {
                    trait_type: "type_2".to_string(),
                    value: "some_value".to_string(),
                },
                Trait {
                    trait_type: "type_3".to_string(),
                    value: "Banana".to_string(),
                },
            ];

            let msg = QueryMsg::Metadata { token_id: 1 };
            let res: ResponseWrapper<MetadataResponse> = app
                .wrap()
                .query_wasm_smart(metadata_contract_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.metadata.attributes, new_attributes);
            let res: ResponseWrapper<MetadataResponse> = app
                .wrap()
                .query_wasm_smart(metadata_contract_addr_2.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.metadata.attributes, new_attributes);
            let res: ResponseWrapper<MetadataResponse> = app
                .wrap()
                .query_wasm_smart(metadata_contract_addr_3.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.metadata.attributes, new_attributes);
        }

        #[test]
        fn test_existing_attribute() {
            let mut app = mock_app();
            let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::OneToOne);
            let metadata_contract_addr_2 = proper_instantiate(&mut app, MetadataType::Static);
            let metadata_contract_addr_3 = proper_instantiate(&mut app, MetadataType::Dynamic);

            setup_metadata(&mut app, metadata_contract_addr.clone());
            setup_metadata(&mut app, metadata_contract_addr_2.clone());
            setup_metadata(&mut app, metadata_contract_addr_3.clone());

            let msg = ExecuteMsg::LinkMetadata {
                token_id: 1,
                metadata_id: Some(1),
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_2.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_3.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let attribute = Trait {
                trait_type: "random".to_string(),
                value: "some_value".to_string(),
            };
            let msg = ExecuteMsg::UpdateAttribute {
                token_id: 1,
                attribute: attribute.clone(),
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AttributeNotFound {}.to_string()
            );
            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_2.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AttributeNotFound {}.to_string()
            );
            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_3.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AttributeNotFound {}.to_string()
            );
        }

        #[test]
        fn test_missing_metadata() {
            let mut app = mock_app();
            let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::OneToOne);
            let metadata_contract_addr_2 = proper_instantiate(&mut app, MetadataType::Static);
            let metadata_contract_addr_3 = proper_instantiate(&mut app, MetadataType::Dynamic);

            let attribute = Trait {
                trait_type: "new_trait".to_string(),
                value: "some_value".to_string(),
            };
            let msg = ExecuteMsg::UpdateAttribute {
                token_id: 1,
                attribute: attribute.clone(),
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MissingMetadata {}.to_string()
            );
            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_2.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MissingMetadata {}.to_string()
            );
            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_3.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MissingMetadata {}.to_string()
            );
        }
    }

    mod remove_attribute {
        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::OneToOne);
            let metadata_contract_addr_2 = proper_instantiate(&mut app, MetadataType::Static);
            let metadata_contract_addr_3 = proper_instantiate(&mut app, MetadataType::Dynamic);

            setup_metadata(&mut app, metadata_contract_addr.clone());
            setup_metadata(&mut app, metadata_contract_addr_2.clone());
            setup_metadata(&mut app, metadata_contract_addr_3.clone());

            let msg = ExecuteMsg::LinkMetadata {
                token_id: 1,
                metadata_id: Some(1),
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_2.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_3.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let msg = ExecuteMsg::RemoveAttribute {
                token_id: 1,
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
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_2.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_3.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let new_attributes = vec![
                Trait {
                    trait_type: "type_1".to_string(),
                    value: "10".to_string(),
                },
                Trait {
                    trait_type: "type_3".to_string(),
                    value: "Banana".to_string(),
                },
            ];

            let msg = QueryMsg::Metadata { token_id: 1 };
            let res: ResponseWrapper<MetadataResponse> = app
                .wrap()
                .query_wasm_smart(metadata_contract_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.metadata.attributes, new_attributes);
            let res: ResponseWrapper<MetadataResponse> = app
                .wrap()
                .query_wasm_smart(metadata_contract_addr_2.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.metadata.attributes, new_attributes);
            let res: ResponseWrapper<MetadataResponse> = app
                .wrap()
                .query_wasm_smart(metadata_contract_addr_3.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.metadata.attributes, new_attributes);
        }

        #[test]
        fn test_existing_attribute() {
            let mut app = mock_app();
            let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::OneToOne);
            let metadata_contract_addr_2 = proper_instantiate(&mut app, MetadataType::Static);
            let metadata_contract_addr_3 = proper_instantiate(&mut app, MetadataType::Dynamic);

            setup_metadata(&mut app, metadata_contract_addr.clone());
            setup_metadata(&mut app, metadata_contract_addr_2.clone());
            setup_metadata(&mut app, metadata_contract_addr_3.clone());

            let msg = ExecuteMsg::LinkMetadata {
                token_id: 1,
                metadata_id: Some(1),
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_2.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_3.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap();

            let msg = ExecuteMsg::RemoveAttribute {
                token_id: 1,
                trait_type: "random_type".to_string(),
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AttributeNotFound {}.to_string()
            );
            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_2.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AttributeNotFound {}.to_string()
            );
            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_3.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AttributeNotFound {}.to_string()
            );
        }

        #[test]
        fn test_missing_metadata() {
            let mut app = mock_app();
            let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::OneToOne);
            let metadata_contract_addr_2 = proper_instantiate(&mut app, MetadataType::Static);
            let metadata_contract_addr_3 = proper_instantiate(&mut app, MetadataType::Dynamic);

            let msg = ExecuteMsg::RemoveAttribute {
                token_id: 1,
                trait_type: "new_trait".to_string(),
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MissingMetadata {}.to_string()
            );
            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_2.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MissingMetadata {}.to_string()
            );
            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_contract_addr_3.clone(),
                    &msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MissingMetadata {}.to_string()
            );
        }
    }
}
mod queries {
    use super::*;

    mod raw_metadatas {
        use super::*;

        #[test]
        fn test_normal() {
            let mut app = mock_app();
            let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::Static);

            let meta_info = MetaInfo {
                image: Some("https://example.com/image.png".to_string()),
                external_url: None,
                description: None,
                animation_url: None,
                youtube_url: None,
            };

            for index in 0..50 {
                let attributes = vec![Trait {
                    trait_type: format!("trait_type_{}", index + 1),
                    value: "10".to_string(),
                }];
                let msg = ExecuteMsg::AddMetadata {
                    meta_info: meta_info.clone(),
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

            let msg: QueryMsg = QueryMsg::RawMetadatas {
                start_after: None,
                limit: None,
            };
            let res: ResponseWrapper<Vec<Metadata>> = app
                .wrap()
                .query_wasm_smart(metadata_contract_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.len(), 30);
            assert_eq!(
                res.data[14],
                Metadata {
                    meta_info,
                    attributes: vec![Trait {
                        trait_type: "trait_type_15".to_string(),
                        value: "10".to_string(),
                    }]
                }
            );
        }

        #[test]
        fn test_filters() {
            let mut app = mock_app();
            let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::Static);

            let meta_info = MetaInfo {
                image: Some("https://example.com/image.png".to_string()),
                external_url: None,
                description: None,
                animation_url: None,
                youtube_url: None,
            };

            for index in 0..50 {
                let attributes = vec![Trait {
                    trait_type: format!("trait_type_{}", index + 1),
                    value: "10".to_string(),
                }];
                let msg = ExecuteMsg::AddMetadata {
                    meta_info: meta_info.clone(),
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

            let msg: QueryMsg = QueryMsg::RawMetadatas {
                start_after: Some(35),
                limit: None,
            };
            let res: ResponseWrapper<Vec<Metadata>> = app
                .wrap()
                .query_wasm_smart(metadata_contract_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.len(), 15);
            assert_eq!(
                res.data[0],
                Metadata {
                    meta_info: meta_info.clone(),
                    attributes: vec![Trait {
                        trait_type: "trait_type_36".to_string(),
                        value: "10".to_string(),
                    }]
                }
            );

            let msg: QueryMsg = QueryMsg::RawMetadatas {
                start_after: Some(35),
                limit: Some(7),
            };
            let res: ResponseWrapper<Vec<Metadata>> = app
                .wrap()
                .query_wasm_smart(metadata_contract_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.len(), 7);
            assert_eq!(
                res.data[6],
                Metadata {
                    meta_info,
                    attributes: vec![Trait {
                        trait_type: "trait_type_42".to_string(),
                        value: "10".to_string(),
                    }]
                }
            );
        }
    }

    mod metadatas {
        use super::*;

        mod one_to_one_and_static {
            use super::*;

            #[test]
            fn test_normal() {
                let mut app = mock_app();
                let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::Static);

                let meta_info = MetaInfo {
                    image: Some("https://example.com/image.png".to_string()),
                    external_url: None,
                    description: None,
                    animation_url: None,
                    youtube_url: None,
                };

                for index in 0..50 {
                    let attributes = vec![Trait {
                        trait_type: format!("trait_type_{}", index + 1),
                        value: "10".to_string(),
                    }];
                    let msg = ExecuteMsg::AddMetadata {
                        meta_info: meta_info.clone(),
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

                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 4,
                    metadata_id: Some(37),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();
                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 28,
                    metadata_id: Some(14),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();
                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 53,
                    metadata_id: Some(46),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();
                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 58,
                    metadata_id: Some(34),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg: QueryMsg = QueryMsg::Metadatas {
                    start_after: None,
                    limit: None,
                };
                let res: ResponseWrapper<Vec<MetadataResponse>> = app
                    .wrap()
                    .query_wasm_smart(metadata_contract_addr.clone(), &msg)
                    .unwrap();
                assert_eq!(res.data.len(), 4);
                assert_eq!(
                    res.data[0],
                    MetadataResponse {
                        metadata_id: 4,
                        metadata: Metadata {
                            meta_info: meta_info.clone(),
                            attributes: vec![Trait {
                                trait_type: "trait_type_37".to_string(),
                                value: "10".to_string(),
                            }]
                        }
                    }
                );
                assert_eq!(
                    res.data[2],
                    MetadataResponse {
                        metadata_id: 53,
                        metadata: Metadata {
                            meta_info: meta_info.clone(),
                            attributes: vec![Trait {
                                trait_type: "trait_type_46".to_string(),
                                value: "10".to_string(),
                            }]
                        }
                    }
                );
            }

            #[test]
            fn test_filters() {
                let mut app = mock_app();
                let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::Static);

                let meta_info = MetaInfo {
                    image: Some("https://example.com/image.png".to_string()),
                    external_url: None,
                    description: None,
                    animation_url: None,
                    youtube_url: None,
                };

                for index in 0..50 {
                    let attributes = vec![Trait {
                        trait_type: format!("trait_type_{}", index + 1),
                        value: "10".to_string(),
                    }];
                    let msg = ExecuteMsg::AddMetadata {
                        meta_info: meta_info.clone(),
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

                for index in 0..20 {
                    let msg = ExecuteMsg::LinkMetadata {
                        token_id: index + 1,
                        metadata_id: Some(index + 1),
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

                let msg: QueryMsg = QueryMsg::Metadatas {
                    start_after: Some(14),
                    limit: Some(5),
                };
                let res: ResponseWrapper<Vec<MetadataResponse>> = app
                    .wrap()
                    .query_wasm_smart(metadata_contract_addr.clone(), &msg)
                    .unwrap();
                assert_eq!(res.data.len(), 5);
                assert_eq!(
                    res.data[0],
                    MetadataResponse {
                        metadata_id: 15,
                        metadata: Metadata {
                            meta_info: meta_info.clone(),
                            attributes: vec![Trait {
                                trait_type: "trait_type_15".to_string(),
                                value: "10".to_string(),
                            }]
                        }
                    }
                );
                assert_eq!(
                    res.data[4],
                    MetadataResponse {
                        metadata_id: 19,
                        metadata: Metadata {
                            meta_info: meta_info.clone(),
                            attributes: vec![Trait {
                                trait_type: "trait_type_19".to_string(),
                                value: "10".to_string(),
                            }]
                        }
                    }
                );
            }
        }

        mod dynamic {
            use super::*;

            #[test]
            fn test_normal() {
                let mut app = mock_app();
                let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::Dynamic);

                let meta_info = MetaInfo {
                    image: Some("https://example.com/image.png".to_string()),
                    external_url: None,
                    description: None,
                    animation_url: None,
                    youtube_url: None,
                };

                for index in 0..50 {
                    let attributes = vec![Trait {
                        trait_type: format!("trait_type_{}", index + 1),
                        value: "10".to_string(),
                    }];
                    let msg = ExecuteMsg::AddMetadata {
                        meta_info: meta_info.clone(),
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

                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 4,
                    metadata_id: Some(37),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();
                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 28,
                    metadata_id: Some(14),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();
                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 53,
                    metadata_id: Some(46),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();
                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 58,
                    metadata_id: Some(34),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_contract_addr.clone(),
                        &msg,
                        &vec![],
                    )
                    .unwrap();

                let msg: QueryMsg = QueryMsg::Metadatas {
                    start_after: None,
                    limit: None,
                };
                let res: ResponseWrapper<Vec<MetadataResponse>> = app
                    .wrap()
                    .query_wasm_smart(metadata_contract_addr.clone(), &msg)
                    .unwrap();
                assert_eq!(res.data.len(), 4);
                assert_eq!(
                    res.data[0],
                    MetadataResponse {
                        metadata_id: 4,
                        metadata: Metadata {
                            meta_info: meta_info.clone(),
                            attributes: vec![Trait {
                                trait_type: "trait_type_37".to_string(),
                                value: "10".to_string(),
                            }]
                        }
                    }
                );
                assert_eq!(
                    res.data[2],
                    MetadataResponse {
                        metadata_id: 53,
                        metadata: Metadata {
                            meta_info: meta_info.clone(),
                            attributes: vec![Trait {
                                trait_type: "trait_type_46".to_string(),
                                value: "10".to_string(),
                            }]
                        }
                    }
                );
            }

            #[test]
            fn test_filters() {
                let mut app = mock_app();
                let metadata_contract_addr = proper_instantiate(&mut app, MetadataType::Dynamic);

                let meta_info = MetaInfo {
                    image: Some("https://example.com/image.png".to_string()),
                    external_url: None,
                    description: None,
                    animation_url: None,
                    youtube_url: None,
                };

                for index in 0..50 {
                    let attributes = vec![Trait {
                        trait_type: format!("trait_type_{}", index + 1),
                        value: "10".to_string(),
                    }];
                    let msg = ExecuteMsg::AddMetadata {
                        meta_info: meta_info.clone(),
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

                for index in 0..20 {
                    let msg = ExecuteMsg::LinkMetadata {
                        token_id: index + 1,
                        metadata_id: Some(index + 1),
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

                let msg: QueryMsg = QueryMsg::Metadatas {
                    start_after: Some(14),
                    limit: Some(5),
                };
                let res: ResponseWrapper<Vec<MetadataResponse>> = app
                    .wrap()
                    .query_wasm_smart(metadata_contract_addr.clone(), &msg)
                    .unwrap();
                assert_eq!(res.data.len(), 5);
                assert_eq!(
                    res.data[0],
                    MetadataResponse {
                        metadata_id: 15,
                        metadata: Metadata {
                            meta_info: meta_info.clone(),
                            attributes: vec![Trait {
                                trait_type: "trait_type_15".to_string(),
                                value: "10".to_string(),
                            }]
                        }
                    }
                );
                assert_eq!(
                    res.data[4],
                    MetadataResponse {
                        metadata_id: 19,
                        metadata: Metadata {
                            meta_info: meta_info.clone(),
                            attributes: vec![Trait {
                                trait_type: "trait_type_19".to_string(),
                                value: "10".to_string(),
                            }]
                        }
                    }
                );
            }
        }
    }
}
