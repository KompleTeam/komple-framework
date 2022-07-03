#[cfg(test)]
mod tests {
    use crate::{
        msg::{ExecuteMsg, InstantiateMsg},
        state::CollectionInfo,
    };
    use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg, Empty, StdResult, Uint128, WasmMsg};
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

    pub fn get_cosmos_msg<T: Into<ExecuteMsg>>(addr: String, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: addr,
            msg,
            funds: vec![],
        }
        .into())
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
        use crate::msg::ExecuteMsg;

        #[test]
        fn test_happy_path() {
            let (mut app, minter_addr) = proper_instantiate();

            let msg = ExecuteMsg::Mint {
                recipient: Some(USER.to_string()),
            };
            let res = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr, &msg, &vec![])
                .unwrap();
        }
    }
}
