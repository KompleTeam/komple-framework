use cosmwasm_std::{Addr, Coin, Decimal, Empty, to_binary, Uint128};
use cw721_base::msg::ExecuteMsg as Cw721ExecuteMsg;
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use komple_fee_module::msg::ExecuteMsg as FeeModuleExecuteMsg;
use komple_hub_module::{
    msg::{
        ExecuteMsg as HubExecuteMsg, InstantiateMsg as HubInstantiateMsg, QueryMsg as HubQueryMsg,
    },
    state::HubInfo,
};
use komple_marketplace_module::msg::{ExecuteMsg, InstantiateMsg, MarketplaceFundInfo};
use komple_metadata_module::msg::InstantiateMsg as MetadataInstantiateMsg;
use komple_mint_module::{
    msg::{CollectionFundInfo, ExecuteMsg as MintExecuteMsg},
    state::CollectionInfo,
};
use komple_token_module::msg::{ExecuteMsg as TokenExecuteMsg, MetadataInfo, TokenInfo};
use komple_token_module::state::CollectionConfig;
use komple_types::modules::metadata::Metadata as MetadataType;
use komple_types::modules::Modules;
use komple_types::query::ResponseWrapper;
use komple_types::shared::RegisterMsg;
use komple_types::modules::mint::Collections;
use komple_utils::storage::StorageHelper;
use std::str::FromStr;
use komple_types::modules::fee::{MarketplaceFees, MintFees};
use komple_types::modules::fee::{Fees, PercentagePayment as FeeModulePercentagePayment};

pub const CREATOR: &str = "juno..creator";
pub const USER: &str = "juno..user";
pub const RANDOM: &str = "juno..random";
pub const ADMIN: &str = "juno..admin";
pub const RANDOM_2: &str = "juno..random2";
pub const NATIVE_DENOM: &str = "native_denom";
pub const TEST_DENOM: &str = "test_denom";

pub fn hub_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_hub_module::contract::execute,
        komple_hub_module::contract::instantiate,
        komple_hub_module::contract::query,
    )
    .with_reply(komple_hub_module::contract::reply);
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

pub fn token_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_token_module::contract::execute,
        komple_token_module::contract::instantiate,
        komple_token_module::contract::query,
    )
    .with_reply(komple_token_module::contract::reply);
    Box::new(contract)
}

pub fn marketplace_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_marketplace_module::contract::execute,
        komple_marketplace_module::contract::instantiate,
        komple_marketplace_module::contract::query,
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

pub fn fee_module() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        komple_fee_module::contract::execute,
        komple_fee_module::contract::instantiate,
        komple_fee_module::contract::query,
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

pub fn setup_fee_module(app: &mut App, fee_module_addr: &Addr) {
    // Komple is 4%
    let msg = FeeModuleExecuteMsg::SetFee {
        fee_type: Fees::Percentage,
        module_name: Modules::Marketplace.to_string(),
        fee_name: MarketplaceFees::Komple.as_str().to_string(),
        data: to_binary(&FeeModulePercentagePayment {
            address: Some("contract0".to_string()),
            value: Decimal::from_str("0.04").unwrap(),
        })
        .unwrap(),
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), fee_module_addr.clone(), &msg, &[])
        .unwrap();
    // Community pool is 2%
    let msg = FeeModuleExecuteMsg::SetFee {
        fee_type: Fees::Percentage,
        module_name: Modules::Marketplace.to_string(),
        fee_name: MarketplaceFees::Community.as_str().to_string(),
        data: to_binary(&FeeModulePercentagePayment {
            address: Some("juno..community".to_string()),
            value: Decimal::from_str("0.02").unwrap(),
        })
        .unwrap(),
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), fee_module_addr.clone(), &msg, &[])
        .unwrap();
    // Hub owner is 2%
    let msg = FeeModuleExecuteMsg::SetFee {
        fee_type: Fees::Percentage,
        module_name: Modules::Marketplace.to_string(),
        fee_name: MarketplaceFees::HubAdmin.as_str().to_string(),
        data: to_binary(&FeeModulePercentagePayment {
            address: None,
            value: Decimal::from_str("0.02").unwrap(),
        })
        .unwrap(),
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), fee_module_addr.clone(), &msg, &[])
        .unwrap();
}

pub fn set_royalties(app: &mut App, fee_module_addr: &Addr, collection_id: u32, royalty: &str) {
    let msg = FeeModuleExecuteMsg::SetFee {
        fee_type: Fees::Percentage,
        module_name: Modules::Mint.to_string(),
        fee_name: MintFees::new_royalty(collection_id),
        data: to_binary(&FeeModulePercentagePayment {
            address: None,
            value: Decimal::from_str(royalty).unwrap(),
        })
        .unwrap(),
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), fee_module_addr.clone(), &msg, &[])
        .unwrap();
}

pub fn setup_hub_module(app: &mut App, is_marbu: bool) -> Addr {
    let hub_code_id = app.store_code(hub_module());

    let fee_module_addr = match is_marbu {
        true => {
            let fee_code_id = app.store_code(fee_module());

            let msg = RegisterMsg {
                admin: ADMIN.to_string(),
                data: None,
            };
            let fee_module_addr = app
                .instantiate_contract(fee_code_id, Addr::unchecked(ADMIN), &msg, &[], "test", None)
                .unwrap();

            setup_fee_module(app, &fee_module_addr);

            Some(fee_module_addr.to_string())
        }
        false => None,
    };

    let msg = HubInstantiateMsg {
        hub_info: HubInfo {
            name: "Test Hub".to_string(),
            description: "Test Hub".to_string(),
            image: "https://example.com/image.png".to_string(),
            external_link: None,
        },
        marbu_fee_module: fee_module_addr,
    };
    let register_msg = RegisterMsg {
        admin: ADMIN.to_string(),
        data: Some(to_binary(&msg).unwrap()),
    };

    app.instantiate_contract(
        hub_code_id,
        Addr::unchecked(ADMIN),
        &register_msg,
        &[],
        "test",
        None,
    )
    .unwrap()
}

pub fn setup_modules(app: &mut App, hub_addr: Addr) -> (Addr, Addr) {
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
                is_native: true,
                denom: NATIVE_DENOM.to_string(),
                cw20_address: None,
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

pub fn create_collection(
    app: &mut App,
    mint_module_addr: Addr,
    creator_addr: &str,
    token_module_code_id: u64,
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
        .execute_contract(Addr::unchecked(creator_addr), mint_module_addr, &msg, &[])
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

pub fn setup_token_module_operators(app: &mut App, token_module_addr: Addr, addrs: Vec<String>) {
    let msg: Cw721ExecuteMsg<Empty, TokenExecuteMsg> = Cw721ExecuteMsg::Extension {
        msg: TokenExecuteMsg::UpdateModuleOperators { addrs },
    };
    let _ = app
        .execute_contract(Addr::unchecked(ADMIN), token_module_addr, &msg, &[])
        .unwrap();
}

pub fn give_approval_to_module(
    app: &mut App,
    token_module_addr: Addr,
    owner: &str,
    operator_addr: &Addr,
) {
    let msg: Cw721ExecuteMsg<Empty, TokenExecuteMsg> = Cw721ExecuteMsg::ApproveAll {
        operator: operator_addr.to_string(),
        expires: None,
    };
    let _ = app
        .execute_contract(Addr::unchecked(owner), token_module_addr, &msg, &[])
        .unwrap();
}

pub fn setup_marketplace_listing(
    app: &mut App,
    mint_module_addr: &Addr,
    marketplace_module_addr: &Addr,
    collection_id: u32,
    token_id: u32,
    price: Uint128,
) {
    let collection_addr =
        StorageHelper::query_collection_address(&app.wrap(), mint_module_addr, &collection_id)
            .unwrap();

    setup_token_module_operators(
        app,
        collection_addr,
        vec![marketplace_module_addr.to_string()],
    );

    let msg = ExecuteMsg::ListFixedToken {
        collection_id,
        token_id,
        price,
    };
    let _ = app
        .execute_contract(
            Addr::unchecked(USER),
            marketplace_module_addr.clone(),
            &msg,
            &[],
        )
        .unwrap();
}
