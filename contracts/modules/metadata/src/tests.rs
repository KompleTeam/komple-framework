use crate::msg::MetadataResponse;
use crate::ContractError;
use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{MetaInfo, Metadata, Trait},
};
use cosmwasm_std::{to_binary, Addr, Coin, Empty, Uint128};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_types::modules::metadata::Metadata as MetadataType;
use komple_types::shared::query::ResponseWrapper;
use komple_types::shared::RegisterMsg;

pub fn metadata_module() -> Box<dyn Contract<Empty>> {
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
    let metadata_code_id = app.store_code(metadata_module());

    let msg = RegisterMsg {
        admin: ADMIN.to_string(),
        data: Some(to_binary(&InstantiateMsg { metadata_type }).unwrap()),
    };

    app.instantiate_contract(
        metadata_code_id,
        Addr::unchecked(ADMIN),
        &msg,
        &[],
        "test",
        None,
    )
    .unwrap()
}

fn setup_metadata(app: &mut App, metadata_module_addr: Addr) -> (Vec<Trait>, MetaInfo) {
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
        .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr, &msg, &[])
        .unwrap();
    (attributes, meta_info)
}

mod initialization {
    use super::*;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let metadata_code_id = app.store_code(metadata_module());

        let msg = RegisterMsg {
            admin: ADMIN.to_string(),
            data: Some(
                to_binary(&InstantiateMsg {
                    metadata_type: MetadataType::Shared,
                })
                .unwrap(),
            ),
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
            let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Shared);

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
                animation_url: Some("https://example.com/animation.mp4".to_string()),
                youtube_url: None,
            };
            let msg = ExecuteMsg::AddMetadata {
                meta_info: meta_info.clone(),
                attributes: attributes.clone(),
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_module_addr.clone(),
                    &msg,
                    &[],
                )
                .unwrap();

            let msg = QueryMsg::RawMetadata { metadata_id: 1 };
            let res: ResponseWrapper<Metadata> = app
                .wrap()
                .query_wasm_smart(metadata_module_addr, &msg)
                .unwrap();
            assert_eq!(res.data.attributes, attributes);
            assert_eq!(res.data.meta_info, meta_info);
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Standard);

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
                meta_info: meta_info,
                attributes: attributes,
            };
            let err = app
                .execute_contract(Addr::unchecked(USER), metadata_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }
    }

    mod link_metadata {
        use super::*;

        mod standard_metadata {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Standard);

                let (attributes, meta_info) =
                    setup_metadata(&mut app, metadata_module_addr.clone());

                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 1,
                    metadata_id: None,
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();

                let msg = QueryMsg::Metadata { token_id: 1 };
                let res: ResponseWrapper<MetadataResponse> = app
                    .wrap()
                    .query_wasm_smart(metadata_module_addr, &msg)
                    .unwrap();
                assert_eq!(res.data.metadata.attributes, attributes);
                assert_eq!(res.data.metadata.meta_info, meta_info);
            }
        }

        mod shared_metadata {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Shared);

                let (attributes, meta_info) =
                    setup_metadata(&mut app, metadata_module_addr.clone());

                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 1,
                    metadata_id: Some(1),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();

                let msg = QueryMsg::Metadata { token_id: 1 };
                let res: ResponseWrapper<MetadataResponse> = app
                    .wrap()
                    .query_wasm_smart(metadata_module_addr, &msg)
                    .unwrap();
                assert_eq!(res.data.metadata.attributes, attributes);
                assert_eq!(res.data.metadata.meta_info, meta_info);
            }

            #[test]
            fn test_missing_metadata() {
                let mut app = mock_app();
                let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Shared);

                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 1,
                    metadata_id: None,
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::MissingMetadata {}.to_string()
                )
            }
        }

        mod dynamic_metadata {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Dynamic);

                let (attributes, meta_info) =
                    setup_metadata(&mut app, metadata_module_addr.clone());

                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 1,
                    metadata_id: Some(1),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();

                let msg = QueryMsg::Metadata { token_id: 1 };
                let res: ResponseWrapper<MetadataResponse> = app
                    .wrap()
                    .query_wasm_smart(metadata_module_addr, &msg)
                    .unwrap();
                assert_eq!(res.data.metadata.attributes, attributes);
                assert_eq!(res.data.metadata.meta_info, meta_info);
            }

            #[test]
            fn test_missing_metadata() {
                let mut app = mock_app();
                let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Dynamic);

                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 1,
                    metadata_id: None,
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr, &msg, &[])
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
            let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Standard);

            let msg = ExecuteMsg::LinkMetadata {
                token_id: 1,
                metadata_id: None,
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MissingMetadata {}.to_string()
            );
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Standard);

            let msg = ExecuteMsg::LinkMetadata {
                token_id: 1,
                metadata_id: None,
            };
            let err = app
                .execute_contract(Addr::unchecked(USER), metadata_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }
    }

    mod update_meta_info {
        use super::*;

        mod standard_and_shared_metadata {
            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Standard);
                let metadata_module_addr_2 = proper_instantiate(&mut app, MetadataType::Shared);

                setup_metadata(&mut app, metadata_module_addr.clone());
                setup_metadata(&mut app, metadata_module_addr_2.clone());

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
                        metadata_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_module_addr_2.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();

                let msg = ExecuteMsg::UpdateMetaInfo {
                    raw_metadata: false,
                    id: 1,
                    meta_info: new_meta_info.clone(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_module_addr_2.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();

                let msg = QueryMsg::Metadata { token_id: 1 };
                let res: ResponseWrapper<MetadataResponse> = app
                    .wrap()
                    .query_wasm_smart(metadata_module_addr, &msg)
                    .unwrap();
                assert_eq!(res.data.metadata.meta_info, new_meta_info);
                let res: ResponseWrapper<MetadataResponse> = app
                    .wrap()
                    .query_wasm_smart(metadata_module_addr_2, &msg)
                    .unwrap();
                assert_eq!(res.data.metadata.meta_info, new_meta_info);
            }

            #[test]
            fn test_missing_metadata() {
                let mut app = mock_app();
                let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Standard);
                let metadata_module_addr_2 = proper_instantiate(&mut app, MetadataType::Shared);

                let new_meta_info = MetaInfo {
                    image: Some("https://test".to_string()),
                    description: Some("test".to_string()),
                    external_url: Some("https://test".to_string()),
                    animation_url: Some("https://test".to_string()),
                    youtube_url: Some("https://test".to_string()),
                };

                let msg = ExecuteMsg::UpdateMetaInfo {
                    raw_metadata: false,
                    id: 1,
                    meta_info: new_meta_info,
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr, &msg, &[])
                    .unwrap_err();
                assert_eq!(
                    err.source().unwrap().to_string(),
                    ContractError::MissingMetadata {}.to_string()
                );
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr_2, &msg, &[])
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
                let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Dynamic);

                setup_metadata(&mut app, metadata_module_addr.clone());

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
                        metadata_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();

                let msg = ExecuteMsg::UpdateMetaInfo {
                    raw_metadata: false,
                    id: 1,
                    meta_info: new_meta_info.clone(),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();

                let msg = QueryMsg::Metadata { token_id: 1 };
                let res: ResponseWrapper<MetadataResponse> = app
                    .wrap()
                    .query_wasm_smart(metadata_module_addr, &msg)
                    .unwrap();
                assert_eq!(res.data.metadata.meta_info, new_meta_info);
            }

            #[test]
            fn test_missing_metadata() {
                let mut app = mock_app();
                let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Dynamic);

                let new_meta_info = MetaInfo {
                    image: Some("https://test".to_string()),
                    description: Some("test".to_string()),
                    external_url: Some("https://test".to_string()),
                    animation_url: Some("https://test".to_string()),
                    youtube_url: Some("https://test".to_string()),
                };

                let msg = ExecuteMsg::UpdateMetaInfo {
                    raw_metadata: false,
                    id: 1,
                    meta_info: new_meta_info,
                };
                let err = app
                    .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr, &msg, &[])
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
            let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Standard);

            let new_meta_info = MetaInfo {
                image: Some("https://test".to_string()),
                description: Some("test".to_string()),
                external_url: Some("https://test".to_string()),
                animation_url: Some("https://test".to_string()),
                youtube_url: Some("https://test".to_string()),
            };

            let msg = ExecuteMsg::UpdateMetaInfo {
                raw_metadata: false,
                id: 1,
                meta_info: new_meta_info,
            };
            let err = app
                .execute_contract(Addr::unchecked(USER), metadata_module_addr, &msg, &[])
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
            let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Standard);
            let metadata_module_addr_2 = proper_instantiate(&mut app, MetadataType::Shared);
            let metadata_module_addr_3 = proper_instantiate(&mut app, MetadataType::Dynamic);

            setup_metadata(&mut app, metadata_module_addr.clone());
            setup_metadata(&mut app, metadata_module_addr_2.clone());
            setup_metadata(&mut app, metadata_module_addr_3.clone());

            let attribute = Trait {
                trait_type: "new_trait".to_string(),
                value: "some_value".to_string(),
            };

            let msg = ExecuteMsg::AddAttribute {
                raw_metadata: true,
                id: 1,
                attribute: attribute.clone(),
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_module_addr.clone(),
                    &msg,
                    &[],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_module_addr_2.clone(),
                    &msg,
                    &[],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_module_addr_3.clone(),
                    &msg,
                    &[],
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
                attribute,
            ];

            let msg = QueryMsg::RawMetadata { metadata_id: 1 };
            let res: ResponseWrapper<Metadata> = app
                .wrap()
                .query_wasm_smart(metadata_module_addr, &msg)
                .unwrap();
            assert_eq!(res.data.attributes, new_attributes);
            let res: ResponseWrapper<Metadata> = app
                .wrap()
                .query_wasm_smart(metadata_module_addr_2, &msg)
                .unwrap();
            assert_eq!(res.data.attributes, new_attributes);
            let res: ResponseWrapper<Metadata> = app
                .wrap()
                .query_wasm_smart(metadata_module_addr_3, &msg)
                .unwrap();
            assert_eq!(res.data.attributes, new_attributes);
        }

        #[test]
        fn test_existing_attribute() {
            let mut app = mock_app();
            let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Standard);
            let metadata_module_addr_2 = proper_instantiate(&mut app, MetadataType::Shared);
            let metadata_module_addr_3 = proper_instantiate(&mut app, MetadataType::Dynamic);

            setup_metadata(&mut app, metadata_module_addr.clone());
            setup_metadata(&mut app, metadata_module_addr_2.clone());
            setup_metadata(&mut app, metadata_module_addr_3.clone());

            let attribute = Trait {
                trait_type: "type_1".to_string(),
                value: "some_value".to_string(),
            };
            let msg = ExecuteMsg::AddAttribute {
                raw_metadata: true,
                id: 1,
                attribute,
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AttributeAlreadyExists {}.to_string()
            );
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr_2, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AttributeAlreadyExists {}.to_string()
            );
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr_3, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AttributeAlreadyExists {}.to_string()
            );
        }

        #[test]
        fn test_missing_metadata() {
            let mut app = mock_app();
            let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Standard);
            let metadata_module_addr_2 = proper_instantiate(&mut app, MetadataType::Shared);
            let metadata_module_addr_3 = proper_instantiate(&mut app, MetadataType::Dynamic);

            let attribute = Trait {
                trait_type: "new_trait".to_string(),
                value: "some_value".to_string(),
            };
            let msg = ExecuteMsg::AddAttribute {
                raw_metadata: false,
                id: 1,
                attribute: attribute,
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MissingMetadata {}.to_string()
            );
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr_2, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MissingMetadata {}.to_string()
            );
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr_3, &msg, &[])
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
            let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Standard);
            let metadata_module_addr_2 = proper_instantiate(&mut app, MetadataType::Shared);
            let metadata_module_addr_3 = proper_instantiate(&mut app, MetadataType::Dynamic);

            setup_metadata(&mut app, metadata_module_addr.clone());
            setup_metadata(&mut app, metadata_module_addr_2.clone());
            setup_metadata(&mut app, metadata_module_addr_3.clone());

            let attribute = Trait {
                trait_type: "type_2".to_string(),
                value: "some_value".to_string(),
            };

            let msg = ExecuteMsg::UpdateAttribute {
                raw_metadata: true,
                id: 1,
                attribute,
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_module_addr.clone(),
                    &msg,
                    &[],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_module_addr_2.clone(),
                    &msg,
                    &[],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_module_addr_3.clone(),
                    &msg,
                    &[],
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

            let msg = QueryMsg::RawMetadata { metadata_id: 1 };
            let res: ResponseWrapper<Metadata> = app
                .wrap()
                .query_wasm_smart(metadata_module_addr, &msg)
                .unwrap();
            assert_eq!(res.data.attributes, new_attributes);
            let res: ResponseWrapper<Metadata> = app
                .wrap()
                .query_wasm_smart(metadata_module_addr_2, &msg)
                .unwrap();
            assert_eq!(res.data.attributes, new_attributes);
            let res: ResponseWrapper<Metadata> = app
                .wrap()
                .query_wasm_smart(metadata_module_addr_3, &msg)
                .unwrap();
            assert_eq!(res.data.attributes, new_attributes);
        }

        #[test]
        fn test_existing_attribute() {
            let mut app = mock_app();
            let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Standard);
            let metadata_module_addr_2 = proper_instantiate(&mut app, MetadataType::Shared);
            let metadata_module_addr_3 = proper_instantiate(&mut app, MetadataType::Dynamic);

            setup_metadata(&mut app, metadata_module_addr.clone());
            setup_metadata(&mut app, metadata_module_addr_2.clone());
            setup_metadata(&mut app, metadata_module_addr_3.clone());

            let attribute = Trait {
                trait_type: "random".to_string(),
                value: "some_value".to_string(),
            };
            let msg = ExecuteMsg::UpdateAttribute {
                raw_metadata: true,
                id: 1,
                attribute,
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AttributeNotFound {}.to_string()
            );
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr_2, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AttributeNotFound {}.to_string()
            );
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr_3, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AttributeNotFound {}.to_string()
            );
        }

        #[test]
        fn test_missing_metadata() {
            let mut app = mock_app();
            let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Standard);
            let metadata_module_addr_2 = proper_instantiate(&mut app, MetadataType::Shared);
            let metadata_module_addr_3 = proper_instantiate(&mut app, MetadataType::Dynamic);

            let attribute = Trait {
                trait_type: "new_trait".to_string(),
                value: "some_value".to_string(),
            };
            let msg = ExecuteMsg::UpdateAttribute {
                raw_metadata: false,
                id: 1,
                attribute,
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MissingMetadata {}.to_string()
            );
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr_2, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MissingMetadata {}.to_string()
            );
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr_3, &msg, &[])
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
            let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Standard);
            let metadata_module_addr_2 = proper_instantiate(&mut app, MetadataType::Shared);
            let metadata_module_addr_3 = proper_instantiate(&mut app, MetadataType::Dynamic);

            setup_metadata(&mut app, metadata_module_addr.clone());
            setup_metadata(&mut app, metadata_module_addr_2.clone());
            setup_metadata(&mut app, metadata_module_addr_3.clone());

            let msg = ExecuteMsg::RemoveAttribute {
                raw_metadata: true,
                id: 1,
                trait_type: "type_2".to_string(),
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_module_addr.clone(),
                    &msg,
                    &[],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_module_addr_2.clone(),
                    &msg,
                    &[],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_module_addr_3.clone(),
                    &msg,
                    &[],
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

            let msg = QueryMsg::RawMetadata { metadata_id: 1 };
            let res: ResponseWrapper<Metadata> = app
                .wrap()
                .query_wasm_smart(metadata_module_addr, &msg)
                .unwrap();
            assert_eq!(res.data.attributes, new_attributes);
            let res: ResponseWrapper<Metadata> = app
                .wrap()
                .query_wasm_smart(metadata_module_addr_2, &msg)
                .unwrap();
            assert_eq!(res.data.attributes, new_attributes);
            let res: ResponseWrapper<Metadata> = app
                .wrap()
                .query_wasm_smart(metadata_module_addr_3, &msg)
                .unwrap();
            assert_eq!(res.data.attributes, new_attributes);
        }

        #[test]
        fn test_existing_attribute() {
            let mut app = mock_app();
            let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Standard);
            let metadata_module_addr_2 = proper_instantiate(&mut app, MetadataType::Shared);
            let metadata_module_addr_3 = proper_instantiate(&mut app, MetadataType::Dynamic);

            setup_metadata(&mut app, metadata_module_addr.clone());
            setup_metadata(&mut app, metadata_module_addr_2.clone());
            setup_metadata(&mut app, metadata_module_addr_3.clone());

            let msg = ExecuteMsg::RemoveAttribute {
                raw_metadata: true,
                id: 1,
                trait_type: "random_type".to_string(),
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AttributeNotFound {}.to_string()
            );
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr_2, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AttributeNotFound {}.to_string()
            );
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr_3, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::AttributeNotFound {}.to_string()
            );
        }

        #[test]
        fn test_missing_metadata() {
            let mut app = mock_app();
            let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Standard);
            let metadata_module_addr_2 = proper_instantiate(&mut app, MetadataType::Shared);
            let metadata_module_addr_3 = proper_instantiate(&mut app, MetadataType::Dynamic);

            let msg = ExecuteMsg::RemoveAttribute {
                raw_metadata: false,
                id: 1,
                trait_type: "new_trait".to_string(),
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MissingMetadata {}.to_string()
            );
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr_2, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MissingMetadata {}.to_string()
            );
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), metadata_module_addr_3, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MissingMetadata {}.to_string()
            );
        }
    }

    mod update_operators {
        use komple_types::shared::query::ResponseWrapper;

        use super::*;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Standard);

            let msg = ExecuteMsg::UpdateOperators {
                addrs: vec![
                    "juno..first".to_string(),
                    "juno..second".to_string(),
                    "juno..first".to_string(),
                ],
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_module_addr.clone(),
                    &msg,
                    &[],
                )
                .unwrap();

            let msg = QueryMsg::Operators {};
            let res: ResponseWrapper<Vec<String>> = app
                .wrap()
                .query_wasm_smart(metadata_module_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.len(), 2);
            assert_eq!(res.data[0], "juno..first");
            assert_eq!(res.data[1], "juno..second");

            let msg = ExecuteMsg::UpdateOperators {
                addrs: vec!["juno..third".to_string()],
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked("juno..first"),
                    metadata_module_addr.clone(),
                    &msg,
                    &[],
                )
                .unwrap();

            let msg = QueryMsg::Operators {};
            let res: ResponseWrapper<Vec<String>> = app
                .wrap()
                .query_wasm_smart(metadata_module_addr, &msg)
                .unwrap();
            assert_eq!(res.data.len(), 1);
            assert_eq!(res.data[0], "juno..third");
        }

        #[test]
        fn test_invalid_admin() {
            let mut app = mock_app();
            let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Standard);

            let msg = ExecuteMsg::UpdateOperators {
                addrs: vec!["juno..first".to_string(), "juno..second".to_string()],
            };
            let err = app
                .execute_contract(Addr::unchecked(USER), metadata_module_addr, &msg, &[])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );
        }

        #[test]
        fn test_invalid_operator() {
            let mut app = mock_app();
            let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Standard);

            let msg = ExecuteMsg::UpdateOperators {
                addrs: vec!["juno..first".to_string(), "juno..second".to_string()],
            };
            let _ = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    metadata_module_addr.clone(),
                    &msg,
                    &[],
                )
                .unwrap();

            let err = app
                .execute_contract(
                    Addr::unchecked("juno..third"),
                    metadata_module_addr,
                    &msg,
                    &[],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
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
            let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Shared);

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
                        metadata_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();
            }

            let msg: QueryMsg = QueryMsg::RawMetadatas {
                start_after: None,
                limit: None,
            };
            let res: ResponseWrapper<Vec<MetadataResponse>> = app
                .wrap()
                .query_wasm_smart(metadata_module_addr, &msg)
                .unwrap();
            assert_eq!(res.data.len(), 30);
            assert_eq!(
                res.data[14],
                MetadataResponse {
                    metadata: Metadata {
                        meta_info,
                        attributes: vec![Trait {
                            trait_type: "trait_type_15".to_string(),
                            value: "10".to_string(),
                        }]
                    },
                    metadata_id: 15
                }
            );
        }

        #[test]
        fn test_filters() {
            let mut app = mock_app();
            let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Shared);

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
                        metadata_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();
            }

            let msg: QueryMsg = QueryMsg::RawMetadatas {
                start_after: Some(35),
                limit: None,
            };
            let res: ResponseWrapper<Vec<MetadataResponse>> = app
                .wrap()
                .query_wasm_smart(metadata_module_addr.clone(), &msg)
                .unwrap();
            assert_eq!(res.data.len(), 15);
            assert_eq!(
                res.data[0],
                MetadataResponse {
                    metadata: Metadata {
                        meta_info: meta_info.clone(),
                        attributes: vec![Trait {
                            trait_type: "trait_type_36".to_string(),
                            value: "10".to_string(),
                        }]
                    },
                    metadata_id: 36
                }
            );

            let msg: QueryMsg = QueryMsg::RawMetadatas {
                start_after: Some(35),
                limit: Some(7),
            };
            let res: ResponseWrapper<Vec<MetadataResponse>> = app
                .wrap()
                .query_wasm_smart(metadata_module_addr, &msg)
                .unwrap();
            assert_eq!(res.data.len(), 7);
            assert_eq!(
                res.data[6],
                MetadataResponse {
                    metadata: Metadata {
                        meta_info,
                        attributes: vec![Trait {
                            trait_type: "trait_type_42".to_string(),
                            value: "10".to_string(),
                        }]
                    },
                    metadata_id: 42
                }
            );
        }
    }

    mod metadatas {
        use super::*;

        mod standard_and_shared {
            use super::*;

            #[test]
            fn test_normal() {
                let mut app = mock_app();
                let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Shared);

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
                            metadata_module_addr.clone(),
                            &msg,
                            &[],
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
                        metadata_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();
                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 28,
                    metadata_id: Some(14),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();
                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 53,
                    metadata_id: Some(46),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();
                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 58,
                    metadata_id: Some(34),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();

                let msg: QueryMsg = QueryMsg::Metadatas {
                    start_after: None,
                    limit: None,
                };
                let res: ResponseWrapper<Vec<MetadataResponse>> = app
                    .wrap()
                    .query_wasm_smart(metadata_module_addr, &msg)
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
                            meta_info: meta_info,
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
                let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Shared);

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
                            metadata_module_addr.clone(),
                            &msg,
                            &[],
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
                            metadata_module_addr.clone(),
                            &msg,
                            &[],
                        )
                        .unwrap();
                }

                let msg: QueryMsg = QueryMsg::Metadatas {
                    start_after: Some(14),
                    limit: Some(5),
                };
                let res: ResponseWrapper<Vec<MetadataResponse>> = app
                    .wrap()
                    .query_wasm_smart(metadata_module_addr, &msg)
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
                            meta_info: meta_info,
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
                let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Dynamic);

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
                            metadata_module_addr.clone(),
                            &msg,
                            &[],
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
                        metadata_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();
                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 28,
                    metadata_id: Some(14),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();
                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 53,
                    metadata_id: Some(46),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();
                let msg = ExecuteMsg::LinkMetadata {
                    token_id: 58,
                    metadata_id: Some(34),
                };
                let _ = app
                    .execute_contract(
                        Addr::unchecked(ADMIN),
                        metadata_module_addr.clone(),
                        &msg,
                        &[],
                    )
                    .unwrap();

                let msg: QueryMsg = QueryMsg::Metadatas {
                    start_after: None,
                    limit: None,
                };
                let res: ResponseWrapper<Vec<MetadataResponse>> = app
                    .wrap()
                    .query_wasm_smart(metadata_module_addr, &msg)
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
                            meta_info: meta_info,
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
                let metadata_module_addr = proper_instantiate(&mut app, MetadataType::Dynamic);

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
                            metadata_module_addr.clone(),
                            &msg,
                            &[],
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
                            metadata_module_addr.clone(),
                            &msg,
                            &[],
                        )
                        .unwrap();
                }

                let msg: QueryMsg = QueryMsg::Metadatas {
                    start_after: Some(14),
                    limit: Some(5),
                };
                let res: ResponseWrapper<Vec<MetadataResponse>> = app
                    .wrap()
                    .query_wasm_smart(metadata_module_addr, &msg)
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
                            meta_info: meta_info,
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
