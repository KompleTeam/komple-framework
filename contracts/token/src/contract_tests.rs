#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
    use rift_types::collection::Collections;

    use crate::{
        msg::{InstantiateMsg, TokenInfo},
        state::{CollectionInfo, Contracts},
    };

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

    fn proper_instantiate(max_token_limit: Option<u32>) -> (App, Addr) {
        let mut app = mock_app();
        let token_code_id = app.store_code(token_contract());

        let collection_info = CollectionInfo {
            collection_type: Collections::Normal,
            name: "Test Collection".to_string(),
            description: "Test Description".to_string(),
            image: "https://some-image.com".to_string(),
            external_link: None,
        };
        let token_info = TokenInfo {
            symbol: "TTT".to_string(),
            minter: ADMIN.to_string(),
        };
        let contracts = Contracts {
            whitelist: None,
            royalty: None,
            metadata: None,
        };
        let msg = InstantiateMsg {
            admin: ADMIN.to_string(),
            token_info,
            per_address_limit: None,
            start_time: None,
            collection_info,
            max_token_limit,
            contracts,
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

        use crate::{
            msg::{ExecuteMsg, MintedTokenAmountResponse, QueryMsg},
            state::Locks,
            ContractError,
        };
        use cw721::OwnerOfResponse;

        #[test]
        fn test_happy_path() {
            let (mut app, token_addr) = proper_instantiate(None);

            let msg = ExecuteMsg::Mint {
                owner: USER.to_string(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr.clone(), &msg, &vec![])
                .unwrap();

            let msg = QueryMsg::OwnerOf {
                token_id: "1".to_string(),
                include_expired: None,
            };
            let response: OwnerOfResponse = app
                .wrap()
                .query_wasm_smart(token_addr.clone(), &msg)
                .unwrap();
            assert_eq!(response.owner, USER);

            let msg = QueryMsg::MintedTokensPerAddress {
                address: USER.to_string(),
            };
            let res: MintedTokenAmountResponse =
                app.wrap().query_wasm_smart(token_addr, &msg).unwrap();
            assert_eq!(res.amount, 1);
        }

        #[test]
        fn test_unhappy_path() {
            let (mut app, token_addr) = proper_instantiate(None);

            let mint_msg = ExecuteMsg::Mint {
                owner: USER.to_string(),
            };
            let err = app
                .execute_contract(
                    Addr::unchecked(USER),
                    token_addr.clone(),
                    &mint_msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );

            let locks = Locks {
                mint_lock: true,
                burn_lock: false,
                send_lock: false,
                transfer_lock: false,
            };
            let msg = ExecuteMsg::UpdateLocks {
                locks: locks.clone(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr.clone(), &msg, &vec![])
                .unwrap();

            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    token_addr.clone(),
                    &mint_msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MintLocked {}.to_string()
            );

            let msg = ExecuteMsg::UpdateTokenLock {
                token_id: "1".to_string(),
                locks: locks.clone(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr.clone(), &msg, &vec![])
                .unwrap();

            let locks = Locks {
                mint_lock: false,
                burn_lock: false,
                send_lock: false,
                transfer_lock: false,
            };
            let msg = ExecuteMsg::UpdateLocks {
                locks: locks.clone(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr.clone(), &msg, &vec![])
                .unwrap();

            let err = app
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    token_addr.clone(),
                    &mint_msg,
                    &vec![],
                )
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MintLocked {}.to_string()
            );

            // TODO: Add token per address limit test
            // TODO: Add max token amount test
        }

        #[test]
        fn test_max_token_limit() {
            let (mut app, token_addr) = proper_instantiate(Some(2));

            let msg = ExecuteMsg::Mint {
                owner: USER.to_string(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr.clone(), &msg, &vec![])
                .unwrap();
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr.clone(), &msg, &vec![])
                .unwrap();

            let err = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::TokenLimitReached {}.to_string()
            );
        }
    }

    mod locks {
        use super::*;

        use crate::{
            error::ContractError,
            msg::{ExecuteMsg, LocksReponse, QueryMsg},
            state::Locks,
        };

        #[test]
        fn test_app_level_locks_happy_path() {
            let (mut app, token_addr) = proper_instantiate(None);

            let locks = Locks {
                burn_lock: false,
                mint_lock: true,
                transfer_lock: true,
                send_lock: false,
            };
            let msg = ExecuteMsg::UpdateLocks {
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

            let msg = ExecuteMsg::Mint {
                owner: "random".to_string(),
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MintLocked {}.to_string()
            );

            let msg = ExecuteMsg::TransferNft {
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
            let (mut app, token_addr) = proper_instantiate(None);

            let locks = Locks {
                burn_lock: false,
                mint_lock: true,
                transfer_lock: true,
                send_lock: false,
            };
            let msg = ExecuteMsg::UpdateTokenLock {
                token_id: "1".to_string(),
                locks: locks.clone(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr.clone(), &msg, &vec![])
                .unwrap();

            let msg = QueryMsg::TokenLocks {
                token_id: "1".to_string(),
            };
            let response: LocksReponse = app
                .wrap()
                .query_wasm_smart(token_addr.clone(), &msg)
                .unwrap();
            assert_eq!(response.locks, locks);

            let msg = ExecuteMsg::Mint {
                owner: "random".to_string(),
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::MintLocked {}.to_string()
            );

            let msg = ExecuteMsg::TransferNft {
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

    mod per_address_limit {
        use super::*;

        use crate::{msg::ExecuteMsg, ContractError};

        #[test]
        fn test_happy_path() {
            let (mut app, token_addr) = proper_instantiate(None);

            let msg = ExecuteMsg::UpdatePerAddressLimit {
                per_address_limit: Some(1),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr.clone(), &msg, &vec![])
                .unwrap();

            let msg = ExecuteMsg::Mint {
                owner: USER.to_string(),
            };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr.clone(), &msg, &vec![])
                .unwrap();

            let msg = ExecuteMsg::Mint {
                owner: USER.to_string(),
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::TokenLimitReached {}.to_string()
            );
        }

        #[test]
        fn test_unhappy_path() {
            let (mut app, token_addr) = proper_instantiate(None);

            let msg = ExecuteMsg::UpdatePerAddressLimit {
                per_address_limit: Some(1),
            };
            let err = app
                .execute_contract(Addr::unchecked(USER), token_addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::Unauthorized {}.to_string()
            );

            let msg = ExecuteMsg::UpdatePerAddressLimit {
                per_address_limit: Some(0),
            };
            let err = app
                .execute_contract(Addr::unchecked(ADMIN), token_addr.clone(), &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::InvalidPerAddressLimit {}.to_string()
            );
        }
    }
}
