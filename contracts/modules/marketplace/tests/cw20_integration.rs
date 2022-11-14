use cosmwasm_std::{to_binary, Addr, Empty, Uint128};
use cw20::Cw20Coin;
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
use komple_framework_hub_module::msg::ExecuteMsg as HubExecuteMsg;
use komple_framework_hub_module::msg::QueryMsg as HubQueryMsg;
use komple_framework_marketplace_module::msg::ExecuteMsg as MarketplaceExecuteMsg;
use komple_framework_marketplace_module::msg::{InstantiateMsg, MarketplaceFundInfo};
use komple_framework_mint_module::msg::ExecuteMsg as MintExecuteMsg;
use komple_framework_types::modules::marketplace::Listing;
use komple_framework_types::shared::query::ResponseWrapper;
use komple_framework_types::shared::RegisterMsg;
use komple_utils::funds::FundsError;
use komple_utils::storage::StorageHelper;

pub mod helpers;
use helpers::*;
use komple_framework_types::modules::Modules;

pub fn cw20_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

pub const CW20_DENOM: &str = "cwdenom";

fn setup_cw20_token(app: &mut App) -> Addr {
    let code_id = app.store_code(cw20_contract());
    let msg = Cw20InstantiateMsg {
        name: "Test token".to_string(),
        symbol: CW20_DENOM.to_string(),
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: RANDOM.to_string(),
            amount: Uint128::new(1_000_000),
        }],
        mint: None,
        marketing: None,
    };
    app.instantiate_contract(code_id, Addr::unchecked(ADMIN), &msg, &[], "test", None)
        .unwrap()
}

pub fn setup_modules(app: &mut App, hub_addr: Addr, cw20_addr: Addr) -> (Addr, Addr) {
    let mint_code_id = app.store_code(mint_module());
    let marketplace_code_id = app.store_code(marketplace_module());

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
    let instantiate_msg = Some(
        to_binary(&InstantiateMsg {
            fund_info: MarketplaceFundInfo {
                is_native: false,
                denom: CW20_DENOM.to_string(),
                cw20_address: Some(cw20_addr.to_string()),
            },
        })
        .unwrap(),
    );
    let msg = HubExecuteMsg::RegisterModule {
        module: Modules::Marketplace.to_string(),
        msg: instantiate_msg,
        code_id: marketplace_code_id,
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
        module: Modules::Marketplace.to_string(),
    };
    let marketplace_res: ResponseWrapper<Addr> =
        app.wrap().query_wasm_smart(hub_addr, &msg).unwrap();

    (mint_res.data, marketplace_res.data)
}

mod initialization {
    use super::*;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let hub_addr = setup_hub_module(&mut app, false);
        let marketplace_module_code_id = app.store_code(marketplace_module());

        let cw20_addr = setup_cw20_token(&mut app);

        let instantiate_msg = Some(
            to_binary(&InstantiateMsg {
                fund_info: MarketplaceFundInfo {
                    is_native: false,
                    denom: CW20_DENOM.to_string(),
                    cw20_address: Some(cw20_addr.to_string()),
                },
            })
            .unwrap(),
        );
        let msg = HubExecuteMsg::RegisterModule {
            module: Modules::Marketplace.to_string(),
            msg: instantiate_msg,
            code_id: marketplace_module_code_id,
        };
        let _ = app.execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &[]);

        let res = StorageHelper::query_module_address(
            &app.wrap(),
            &hub_addr,
            Modules::Marketplace.to_string(),
        )
        .unwrap();
        assert_eq!(res, "contract2");
    }

    #[test]
    fn test_invalid_cw20_token() {
        let mut app = mock_app();
        let hub_addr = setup_hub_module(&mut app, false);
        let marketplace_module_code_id = app.store_code(marketplace_module());

        let cw20_addr = setup_cw20_token(&mut app);

        let instantiate_msg = Some(
            to_binary(&InstantiateMsg {
                fund_info: MarketplaceFundInfo {
                    is_native: false,
                    denom: "invalid".to_string(),
                    cw20_address: Some(cw20_addr.to_string()),
                },
            })
            .unwrap(),
        );
        let msg = HubExecuteMsg::RegisterModule {
            module: Modules::Marketplace.to_string(),
            msg: instantiate_msg,
            code_id: marketplace_module_code_id,
        };
        let err = app
            .execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &[])
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().source().unwrap().to_string(),
            FundsError::InvalidCw20Token {}.to_string()
        );

        let instantiate_msg = Some(
            to_binary(&InstantiateMsg {
                fund_info: MarketplaceFundInfo {
                    is_native: false,
                    denom: CW20_DENOM.to_string(),
                    cw20_address: None,
                },
            })
            .unwrap(),
        );
        let msg = HubExecuteMsg::RegisterModule {
            module: Modules::Marketplace.to_string(),
            msg: instantiate_msg,
            code_id: marketplace_module_code_id,
        };
        let err = app
            .execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &[])
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().source().unwrap().to_string(),
            FundsError::InvalidCw20Token {}.to_string()
        );
    }
}

mod actions {
    use super::*;

    mod buying {
        use super::*;

        mod fixed_tokens {

            use super::*;

            #[test]
            fn test_happy_path() {
                let mut app = mock_app();
                let hub_addr = setup_hub_module(&mut app, true);

                let cw20_addr = setup_cw20_token(&mut app);

                let (mint_module_addr, marketplace_module_addr) =
                    setup_modules(&mut app, hub_addr.clone(), cw20_addr.clone());

                // Update public permission settings
                // Creator will be creating the collection
                let msg = MintExecuteMsg::UpdatePublicCollectionCreation {
                    public_collection_creation: true,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), mint_module_addr.clone(), &msg, &[])
                    .unwrap();

                let token_module_code_id = app.store_code(token_module());
                create_collection(
                    &mut app,
                    mint_module_addr.clone(),
                    CREATOR,
                    token_module_code_id,
                );

                let collection_addr =
                    StorageHelper::query_collection_address(&app.wrap(), &mint_module_addr, &1)
                        .unwrap();

                mint_token(&mut app, mint_module_addr.clone(), 1, USER);
                mint_token(&mut app, mint_module_addr.clone(), 1, USER);
                mint_token(&mut app, mint_module_addr.clone(), 1, USER);

                give_approval_to_module(
                    &mut app,
                    collection_addr.clone(),
                    USER,
                    &marketplace_module_addr,
                );

                setup_marketplace_listing(
                    &mut app,
                    &mint_module_addr,
                    &marketplace_module_addr,
                    1,
                    1,
                    Uint128::new(1_000),
                );

                let msg = Cw20ExecuteMsg::Send {
                    contract: marketplace_module_addr.to_string(),
                    amount: Uint128::new(1_000),
                    msg: to_binary(&MarketplaceExecuteMsg::Buy {
                        listing_type: Listing::Fixed,
                        collection_id: 1,
                        token_id: 1,
                    })
                    .unwrap(),
                };
                let _ = app
                    .execute_contract(Addr::unchecked(RANDOM), cw20_addr.clone(), &msg, &[])
                    .unwrap();

                // Buyer balance
                let res: BalanceResponse = app
                    .wrap()
                    .query_wasm_smart(
                        cw20_addr.clone(),
                        &Cw20QueryMsg::Balance {
                            address: RANDOM.to_string(),
                        },
                    )
                    .unwrap();
                assert_eq!(res.balance, Uint128::new(999_000));

                // Owner balance
                let res: BalanceResponse = app
                    .wrap()
                    .query_wasm_smart(
                        cw20_addr.clone(),
                        &Cw20QueryMsg::Balance {
                            address: USER.to_string(),
                        },
                    )
                    .unwrap();
                assert_eq!(res.balance, Uint128::new(920));

                // Komple fee
                let res: BalanceResponse = app
                    .wrap()
                    .query_wasm_smart(
                        cw20_addr.clone(),
                        &Cw20QueryMsg::Balance {
                            address: "contract0".to_string(),
                        },
                    )
                    .unwrap();
                assert_eq!(res.balance, Uint128::new(40));

                // Community fee
                let res: BalanceResponse = app
                    .wrap()
                    .query_wasm_smart(
                        cw20_addr.clone(),
                        &Cw20QueryMsg::Balance {
                            address: "juno..community".to_string(),
                        },
                    )
                    .unwrap();
                assert_eq!(res.balance, Uint128::new(20));

                // Marketplace owner fee
                let res: BalanceResponse = app
                    .wrap()
                    .query_wasm_smart(
                        cw20_addr.clone(),
                        &Cw20QueryMsg::Balance {
                            address: ADMIN.to_string(),
                        },
                    )
                    .unwrap();
                assert_eq!(res.balance, Uint128::new(20));

                let fee_module_code_id = app.store_code(fee_module());
                let msg = HubExecuteMsg::RegisterModule {
                    module: Modules::Fee.to_string(),
                    msg: Some(
                        to_binary(&RegisterMsg {
                            admin: ADMIN.to_string(),
                            data: None,
                        })
                        .unwrap(),
                    ),
                    code_id: fee_module_code_id,
                };
                let _ = app
                    .execute_contract(Addr::unchecked(ADMIN), hub_addr.clone(), &msg, &[])
                    .unwrap();
                let fee_module_addr = StorageHelper::query_module_address(
                    &app.wrap(),
                    &hub_addr,
                    Modules::Fee.to_string(),
                )
                .unwrap();

                // Setup admin royalty for 10 percent
                set_royalties(&mut app, &fee_module_addr, 1, "0.1");

                setup_marketplace_listing(
                    &mut app,
                    &mint_module_addr,
                    &marketplace_module_addr,
                    1,
                    2,
                    Uint128::new(1_000),
                );

                let msg = Cw20ExecuteMsg::Send {
                    contract: marketplace_module_addr.to_string(),
                    amount: Uint128::new(1_000),
                    msg: to_binary(&MarketplaceExecuteMsg::Buy {
                        listing_type: Listing::Fixed,
                        collection_id: 1,
                        token_id: 2,
                    })
                    .unwrap(),
                };
                let _ = app
                    .execute_contract(Addr::unchecked(RANDOM), cw20_addr.clone(), &msg, &[])
                    .unwrap();

                // Buyer balance
                let res: BalanceResponse = app
                    .wrap()
                    .query_wasm_smart(
                        cw20_addr.clone(),
                        &Cw20QueryMsg::Balance {
                            address: RANDOM.to_string(),
                        },
                    )
                    .unwrap();
                assert_eq!(res.balance, Uint128::new(998_000));

                // Owner balance
                let res: BalanceResponse = app
                    .wrap()
                    .query_wasm_smart(
                        cw20_addr.clone(),
                        &Cw20QueryMsg::Balance {
                            address: USER.to_string(),
                        },
                    )
                    .unwrap();
                assert_eq!(res.balance, Uint128::new(1_740));

                // Komple fee
                let res: BalanceResponse = app
                    .wrap()
                    .query_wasm_smart(
                        cw20_addr.clone(),
                        &Cw20QueryMsg::Balance {
                            address: "contract0".to_string(),
                        },
                    )
                    .unwrap();
                assert_eq!(res.balance, Uint128::new(80));

                // Community fee
                let res: BalanceResponse = app
                    .wrap()
                    .query_wasm_smart(
                        cw20_addr.clone(),
                        &Cw20QueryMsg::Balance {
                            address: "juno..community".to_string(),
                        },
                    )
                    .unwrap();
                assert_eq!(res.balance, Uint128::new(40));

                // Marketplace owner
                let res: BalanceResponse = app
                    .wrap()
                    .query_wasm_smart(
                        cw20_addr.clone(),
                        &Cw20QueryMsg::Balance {
                            address: ADMIN.to_string(),
                        },
                    )
                    .unwrap();
                assert_eq!(res.balance, Uint128::new(40));

                // Creator royalty fee
                let res: BalanceResponse = app
                    .wrap()
                    .query_wasm_smart(
                        cw20_addr.clone(),
                        &Cw20QueryMsg::Balance {
                            address: CREATOR.to_string(),
                        },
                    )
                    .unwrap();
                assert_eq!(res.balance, Uint128::new(100));

                // Setup admin royalty for 10 percent
                set_royalties(&mut app, &fee_module_addr, 1, "0.05");

                setup_marketplace_listing(
                    &mut app,
                    &mint_module_addr,
                    &marketplace_module_addr,
                    1,
                    3,
                    Uint128::new(998_000),
                );

                let msg = Cw20ExecuteMsg::Send {
                    contract: marketplace_module_addr.to_string(),
                    amount: Uint128::new(998_000),
                    msg: to_binary(&MarketplaceExecuteMsg::Buy {
                        listing_type: Listing::Fixed,
                        collection_id: 1,
                        token_id: 3,
                    })
                    .unwrap(),
                };
                let _ = app
                    .execute_contract(Addr::unchecked(RANDOM), cw20_addr.clone(), &msg, &[])
                    .unwrap();

                // Buyer balance
                let res: BalanceResponse = app
                    .wrap()
                    .query_wasm_smart(
                        cw20_addr.clone(),
                        &Cw20QueryMsg::Balance {
                            address: RANDOM.to_string(),
                        },
                    )
                    .unwrap();
                assert_eq!(res.balance, Uint128::new(0));

                // Owner balance
                let res: BalanceResponse = app
                    .wrap()
                    .query_wasm_smart(
                        cw20_addr.clone(),
                        &Cw20QueryMsg::Balance {
                            address: USER.to_string(),
                        },
                    )
                    .unwrap();
                assert_eq!(res.balance, Uint128::new(870_000));

                // Komple fee
                let res: BalanceResponse = app
                    .wrap()
                    .query_wasm_smart(
                        cw20_addr.clone(),
                        &Cw20QueryMsg::Balance {
                            address: "contract0".to_string(),
                        },
                    )
                    .unwrap();
                assert_eq!(res.balance, Uint128::new(40_000));

                // Community fee
                let res: BalanceResponse = app
                    .wrap()
                    .query_wasm_smart(
                        cw20_addr.clone(),
                        &Cw20QueryMsg::Balance {
                            address: "juno..community".to_string(),
                        },
                    )
                    .unwrap();
                assert_eq!(res.balance, Uint128::new(20_000));

                // Marketplace owner
                let res: BalanceResponse = app
                    .wrap()
                    .query_wasm_smart(
                        cw20_addr.clone(),
                        &Cw20QueryMsg::Balance {
                            address: ADMIN.to_string(),
                        },
                    )
                    .unwrap();
                assert_eq!(res.balance, Uint128::new(20_000));

                // Creator royalty fee
                let res: BalanceResponse = app
                    .wrap()
                    .query_wasm_smart(
                        cw20_addr.clone(),
                        &Cw20QueryMsg::Balance {
                            address: CREATOR.to_string(),
                        },
                    )
                    .unwrap();
                assert_eq!(res.balance, Uint128::new(50_000));
            }
        }
    }
}
