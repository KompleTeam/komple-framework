#[cfg(test)]
mod tests {
    use crate::msg::{ExecuteMsg, InstantiateMsg};
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
    use token_contract::{msg::TokenInfo, state::CollectionInfo};

    pub fn passcard_module() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply(crate::contract::reply);
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
    const RANDOM: &str = "juno1et88c8yd6xr8azkmp02lxtctkqq36lt63tdt7e";
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
        let passcard_module_code_id = app.store_code(passcard_module());

        let msg = InstantiateMsg {
            admin: ADMIN.to_string(),
        };
        let passcard_module_addr = app
            .instantiate_contract(
                passcard_module_code_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        passcard_module_addr
    }

    fn setup_passcard(app: &mut App, minter_addr: &Addr) {
        let token_code_id = app.store_code(token_contract());

        let collection_info = CollectionInfo {
            name: "Test Passcard".to_string(),
            description: "Test Description".to_string(),
            image: "ipfs://xyz".to_string(),
            external_link: None,
        };
        let token_info = TokenInfo {
            symbol: "TEST".to_string(),
            minter: minter_addr.to_string(),
        };
        let msg = ExecuteMsg::CreatePasscard {
            code_id: token_code_id,
            collection_info,
            token_info,
            per_address_limit: None,
            start_time: None,
            whitelist: None,
            royalty: None,
            main_collections: vec![],
            max_token_limit: Some(2),
        };
        let _ = app
            .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
            .unwrap();
    }

    mod mint {
        use super::*;

        use crate::msg::{AddressResponse, ExecuteMsg, QueryMsg};
        use cw721::OwnerOfResponse;
        use token_contract::msg::QueryMsg as TokenQueryMsg;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let passcard_module_addr = proper_instantiate(&mut app);
            setup_passcard(&mut app, &passcard_module_addr);

            let mint_msg = ExecuteMsg::Mint { passcard_id: 1 };
            let _ = app
                .execute_contract(
                    Addr::unchecked(USER),
                    passcard_module_addr.clone(),
                    &mint_msg,
                    &vec![],
                )
                .unwrap();
            let _ = app
                .execute_contract(
                    Addr::unchecked(RANDOM),
                    passcard_module_addr.clone(),
                    &mint_msg,
                    &vec![],
                )
                .unwrap();

            let msg = QueryMsg::PasscardAddress { passcard_id: 1 };
            let response: AddressResponse = app
                .wrap()
                .query_wasm_smart(passcard_module_addr.clone(), &msg)
                .unwrap();
            let passcard_address = response.address;

            let msg = TokenQueryMsg::OwnerOf {
                token_id: "1".to_string(),
                include_expired: None,
            };
            let response: OwnerOfResponse = app
                .wrap()
                .query_wasm_smart(passcard_address.clone(), &msg)
                .unwrap();
            assert_eq!(response.owner, USER.to_string());

            let msg = TokenQueryMsg::OwnerOf {
                token_id: "2".to_string(),
                include_expired: None,
            };
            let response: OwnerOfResponse =
                app.wrap().query_wasm_smart(passcard_address, &msg).unwrap();
            assert_eq!(response.owner, RANDOM.to_string());
        }
    }
}
