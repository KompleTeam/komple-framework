#[cfg(test)]
mod tests {
    use crate::msg::{ExecuteMsg, InstantiateMsg};
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
    use token_contract::{msg::TokenInfo, state::CollectionInfo};

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
            token_contract::contract::execute,
            token_contract::contract::instantiate,
            token_contract::contract::query,
        );
        Box::new(contract)
    }

    const USER: &str = "juno1shfqtuup76mngspx29gcquykjvvlx9na4kymlm";
    const ADMIN: &str = "juno1qamfln8u5w8d3vlhp5t9mhmylfkgad4jz6t7cv";
    // const RANDOM: &str = "juno1et88c8yd6xr8azkmp02lxtctkqq36lt63tdt7e";
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
        let minter_code_id = app.store_code(minter_contract());

        let msg = InstantiateMsg {
            admin: ADMIN.to_string(),
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

        minter_contract_addr
    }

    fn setup_collection(app: &mut App, minter_addr: &Addr) {
        let token_code_id = app.store_code(token_contract());

        let collection_info = CollectionInfo {
            name: "Test Collection".to_string(),
            description: "Test Description".to_string(),
            image: "ipfs://xyz".to_string(),
            external_link: None,
        };
        let token_info = TokenInfo {
            symbol: "TEST".to_string(),
            minter: minter_addr.to_string(),
        };
        let msg = ExecuteMsg::CreateCollection {
            code_id: token_code_id,
            collection_info,
            token_info,
            per_address_limit: None,
            start_time: None,
            whitelist: None,
            royalty: None,
        };
        let _ = app
            .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
            .unwrap();
    }

    mod mint {
        use super::*;
        use crate::{
            msg::{ExecuteMsg, QueryMsg},
            ContractError,
        };
        use cw721::OwnerOfResponse;
        use rift_types::query::AddressResponse;
        use token_contract::msg::QueryMsg as TokenQueryMsg;

        #[test]
        fn test_happy_path() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);
            setup_collection(&mut app, &minter_addr);

            let msg = ExecuteMsg::Mint { collection_id: 1 };
            let _ = app
                .execute_contract(Addr::unchecked(USER), minter_addr.clone(), &msg, &vec![])
                .unwrap();

            let msg = QueryMsg::CollectionAddress(1);
            let response: AddressResponse = app.wrap().query_wasm_smart(minter_addr, &msg).unwrap();
            let token_address = response.address;

            let msg = TokenQueryMsg::OwnerOf {
                token_id: "1".to_string(),
                include_expired: None,
            };
            let response: OwnerOfResponse =
                app.wrap().query_wasm_smart(token_address, &msg).unwrap();
            assert_eq!(response.owner, USER.to_string());
        }

        #[test]
        fn test_locked_minting() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);
            setup_collection(&mut app, &minter_addr);

            let msg = ExecuteMsg::UpdateMintLock { lock: true };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
                .unwrap();

            let msg = ExecuteMsg::Mint { collection_id: 1 };
            let err = app
                .execute_contract(Addr::unchecked(USER), minter_addr, &msg, &vec![])
                .unwrap_err();
            assert_eq!(
                err.source().unwrap().to_string(),
                ContractError::LockedMint {}.to_string()
            )
        }
    }

    mod locks {
        use super::*;
        use crate::{
            msg::{ExecuteMsg, QueryMsg},
            state::Config,
        };
        use rift_types::query::AddressResponse;
        use token_contract::{
            msg::{LocksReponse, QueryMsg as TokenQueryMsg},
            state::Locks,
        };

        // #[test]
        // fn test_app_level_happy_path() {
        //     let (mut app, minter_addr) = proper_instantiate();

        //     let locks = Locks {
        //         burn_lock: false,
        //         mint_lock: true,
        //         transfer_lock: true,
        //         send_lock: false,
        //     };
        //     let msg = ExecuteMsg::UpdateLocks {
        //         locks: locks.clone(),
        //     };
        //     let _ = app
        //         .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
        //         .unwrap();

        //     let msg = QueryMsg::CollectionAddress { collection_id: 1 };
        //     let response: TokenAddressResponse =
        //         app.wrap().query_wasm_smart(minter_addr, &msg).unwrap();
        //     let token_address = response.token_address;

        //     let msg = TokenQueryMsg::Locks {};
        //     let response: LocksReponse = app.wrap().query_wasm_smart(token_address, &msg).unwrap();
        //     assert_eq!(response.locks, locks);
        // }

        // #[test]
        // fn test_token_level_happy_path() {
        //     let (mut app, minter_addr) = proper_instantiate();

        //     let locks = Locks {
        //         burn_lock: false,
        //         mint_lock: true,
        //         transfer_lock: true,
        //         send_lock: false,
        //     };
        //     let msg = ExecuteMsg::UpdateTokenLocks {
        //         token_id: 1,
        //         locks: locks.clone(),
        //     };
        //     let _ = app
        //         .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
        //         .unwrap();

        //     let msg = QueryMsg::GetTokenAddress {};
        //     let response: TokenAddressResponse =
        //         app.wrap().query_wasm_smart(minter_addr, &msg).unwrap();
        //     let token_address = response.token_address;

        //     let msg = TokenQueryMsg::TokenLocks {
        //         token_id: "1".to_string(),
        //     };
        //     let response: LocksReponse = app.wrap().query_wasm_smart(token_address, &msg).unwrap();
        //     assert_eq!(response.locks, locks);
        // }

        #[test]
        fn test_mint_lock_happy_path() {
            let mut app = mock_app();
            let minter_addr = proper_instantiate(&mut app);

            let msg = ExecuteMsg::UpdateMintLock { lock: true };
            let _ = app
                .execute_contract(Addr::unchecked(ADMIN), minter_addr.clone(), &msg, &vec![])
                .unwrap();

            let msg = QueryMsg::Config {};
            let response: Config = app.wrap().query_wasm_smart(minter_addr, &msg).unwrap();
            assert_eq!(response.mint_lock, true);
        }
    }
}
