#[cfg(test)]
mod tests {
    use crate::{msg::InstantiateMsg, state::CollectionInfo};
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn minter_contract() -> Box<dyn Contract<Empty>> {
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
            token::contract::execute,
            token::contract::instantiate,
            token::contract::query,
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

    fn proper_instantiate() -> (App, Addr) {
        let mut app = mock_app();
        let token_code_id = app.store_code(token_contract());
        let minter_code_id = app.store_code(minter_contract());

        let collection_info = CollectionInfo {
            name: "Test Collection".to_string(),
            description: "Test Description".to_string(),
            image: "https://some-image.com".to_string(),
            external_link: None,
        };
        let msg = InstantiateMsg {
            symbol: "TP".to_string(),
            collection_info,
            token_code_id,
            per_address_limit: None,
            start_time: None,
            whitelist: None,
        };
        let minter_contract_addr = app
            .instantiate_contract(
                minter_code_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        (app, minter_contract_addr)
    }

    mod mint {
        use super::*;
        use crate::msg::{ExecuteMsg, QueryMsg, TokenAddressResponse};
        use cw721::OwnerOfResponse;
        use token::msg::QueryMsg as TokenQueryMsg;

        #[test]
        fn test_happy_path() {
            let (mut app, minter_addr) = proper_instantiate();

            let msg = ExecuteMsg::Mint {
                recipient: Some(USER.to_string()),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
                .unwrap();

            let msg = QueryMsg::GetTokenAddress {};
            let response: TokenAddressResponse =
                app.wrap().query_wasm_smart(minter_addr, &msg).unwrap();
            let token_address = response.token_address;

            let msg = TokenQueryMsg::OwnerOf {
                token_id: "1".to_string(),
                include_expired: None,
            };
            let response: OwnerOfResponse =
                app.wrap().query_wasm_smart(token_address, &msg).unwrap();
            assert_eq!(response.owner, USER.to_string());
        }
    }
}
