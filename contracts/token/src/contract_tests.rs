#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw721_base::msg::InstantiateMsg;
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn token_contract() -> Box<dyn Contract<Empty>> {
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

    fn proper_instantiate() -> (App, Addr) {
        let mut app = mock_app();
        let token_code_id = app.store_code(token_contract());

        let msg = InstantiateMsg {
            name: "token".to_string(),
            symbol: "TOKEN".to_string(),
            minter: ADMIN.to_string(),
        };
        let token_contract_addr = app
            .instantiate_contract(
                token_code_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        (app, token_contract_addr)
    }

    mod mint {
        use super::*;

        use crate::msg::{ExecuteMsg, QueryMsg};
        use cw721::OwnerOfResponse;
        use cw721_base::MintMsg;

        #[test]
        fn test_happy_path() {
            let (mut app, token_addr) = proper_instantiate();

            let msg = ExecuteMsg::Mint(MintMsg {
                token_id: "1".to_string(),
                owner: ADMIN.to_string(),
                token_uri: None,
                extension: Empty {},
            });
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr.clone(), &msg, &vec![])
                .unwrap();

            let msg = QueryMsg::OwnerOf {
                token_id: "1".to_string(),
                include_expired: None,
            };
            let response: OwnerOfResponse = app.wrap().query_wasm_smart(token_addr, &msg).unwrap();
            assert_eq!(response.owner, ADMIN);
        }
    }

    mod locks {
        use super::*;

        use crate::{
            error::ContractError,
            msg::{ExecuteMsg, LocksReponse, QueryMsg, TokenLocksReponse},
            state::Locks,
        };
        use cw721_base::MintMsg;

        #[test]
        fn test_app_level_locks_happy_path() {
            let (mut app, token_addr) = proper_instantiate();

            let locks = Locks {
                burn_lock: false,
                mint_lock: true,
                transfer_lock: true,
                send_lock: false,
            };
            let msg: ExecuteMsg<Empty> = ExecuteMsg::UpdateLocks {
                locks: locks.clone(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr.clone(), &msg, &vec![])
                .unwrap();

            let msg = QueryMsg::Locks {};
            let response: LocksReponse = app
                .wrap()
                .query_wasm_smart(token_addr.clone(), &msg)
                .unwrap();
            assert_eq!(response.locks, locks);

            let msg: ExecuteMsg<Empty> = ExecuteMsg::Mint(MintMsg {
                token_id: "1".to_string(),
                owner: "random".to_string(),
                token_uri: None,
                extension: Empty {},
            });
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MintLocked {}.to_string()
            );

            let msg: ExecuteMsg<Empty> = ExecuteMsg::TransferNft {
                recipient: "admin".to_string(),
                token_id: "1".to_string(),
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr, &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::TransferLocked {}.to_string()
            );
        }

        #[test]
        fn test_token_level_locks_happy_path() {
            let (mut app, token_addr) = proper_instantiate();

            let locks = Locks {
                burn_lock: false,
                mint_lock: true,
                transfer_lock: true,
                send_lock: false,
            };
            let msg: ExecuteMsg<Empty> = ExecuteMsg::UpdateTokenLock {
                token_id: "1".to_string(),
                locks: locks.clone(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr.clone(), &msg, &vec![])
                .unwrap();

            let msg = QueryMsg::TokenLocks {
                token_id: "1".to_string(),
            };
            let response: TokenLocksReponse = app
                .wrap()
                .query_wasm_smart(token_addr.clone(), &msg)
                .unwrap();
            assert_eq!(response.locks, locks);

            let msg: ExecuteMsg<Empty> = ExecuteMsg::Mint(MintMsg {
                token_id: "1".to_string(),
                owner: "random".to_string(),
                token_uri: None,
                extension: Empty {},
            });
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MintLocked {}.to_string()
            );

            let msg: ExecuteMsg<Empty> = ExecuteMsg::TransferNft {
                recipient: "admin".to_string(),
                token_id: "1".to_string(),
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr, &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::TransferLocked {}.to_string()
            );
        }
    }
}
