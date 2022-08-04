use crate::msg::ExecuteMsg;
use cosmwasm_std::Addr;
use cw_multi_test::Executor;

use crate::tests::integration::helpers::{
    create_collection, get_collection_address, get_modules_addresses, give_approval_to_module,
    merge_module, mint_token, mock_app, proper_instantiate, setup_all_modules,
    setup_mint_module_whitelist, token_contract, ADMIN, USER,
};

mod initialization {
    use super::*;

    use rift_types::{module::Modules, query::AddressResponse};

    use crate::{msg::QueryMsg, ContractError};

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let controller_addr = proper_instantiate(&mut app);
        let merge_module_code_id = app.store_code(merge_module());

        let msg = ExecuteMsg::InitMergeModule {
            code_id: merge_module_code_id,
        };
        let _ = app.execute_contract(
            Addr::unchecked(ADMIN),
            controller_addr.clone(),
            &msg,
            &vec![],
        );

        let msg = QueryMsg::ModuleAddress(Modules::MergeModule);
        let res: AddressResponse = app.wrap().query_wasm_smart(controller_addr, &msg).unwrap();
        assert_eq!(res.address, "contract1")
    }

    #[test]
    fn test_invalid_sender() {
        let mut app = mock_app();
        let controller_addr = proper_instantiate(&mut app);
        let merge_module_code_id = app.store_code(merge_module());

        let msg = ExecuteMsg::InitMergeModule {
            code_id: merge_module_code_id,
        };
        let err = app
            .execute_contract(
                Addr::unchecked(USER),
                controller_addr.clone(),
                &msg,
                &vec![],
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            ContractError::Unauthorized {}.to_string()
        )
    }
}

mod normal_merge {
    use super::*;

    use cosmwasm_std::to_binary;
    use cw721::OwnerOfResponse;
    use merge_module::{
        msg::{ExecuteMsg as MergeExecuteMsg, MergeAction, MergeMsg},
        ContractError as MergeContractError,
    };
    use rift_types::collection::Collections;
    use token_contract::msg::QueryMsg as TokenQueryMsg;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let controller_addr = proper_instantiate(&mut app);

        setup_all_modules(&mut app, controller_addr.clone());

        let (mint_module_addr, merge_module_addr, _) =
            get_modules_addresses(&mut app, &controller_addr.to_string());

        let token_contract_code_id = app.store_code(token_contract());
        create_collection(
            &mut app,
            mint_module_addr.clone(),
            token_contract_code_id,
            None,
            None,
            None,
            None,
        );
        create_collection(
            &mut app,
            mint_module_addr.clone(),
            token_contract_code_id,
            None,
            None,
            None,
            None,
        );

        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 1, USER);

        setup_mint_module_whitelist(
            &mut app,
            mint_module_addr.clone(),
            vec![merge_module_addr.to_string()],
        );

        let collection_1_addr = get_collection_address(&mut app, &mint_module_addr.to_string(), 1);
        give_approval_to_module(
            &mut app,
            collection_1_addr.clone(),
            USER,
            &merge_module_addr,
        );

        let merge_msg = vec![
            MergeMsg {
                collection_id: 1,
                token_id: Some(1),
                collection_type: Collections::Normal,
                action: MergeAction::Burn,
            },
            MergeMsg {
                collection_id: 1,
                token_id: Some(3),
                collection_type: Collections::Normal,
                action: MergeAction::Burn,
            },
            MergeMsg {
                collection_id: 2,
                token_id: None,
                collection_type: Collections::Normal,
                action: MergeAction::Mint,
            },
        ];
        let msg = MergeExecuteMsg::Merge {
            msg: to_binary(&merge_msg).unwrap(),
        };
        let _ = app
            .execute_contract(Addr::unchecked(USER), merge_module_addr, &msg, &vec![])
            .unwrap();

        let msg = TokenQueryMsg::OwnerOf {
            token_id: "1".to_string(),
            include_expired: None,
        };
        let res: Result<OwnerOfResponse, cosmwasm_std::StdError> =
            app.wrap().query_wasm_smart(collection_1_addr.clone(), &msg);
        assert!(res.is_err());

        let msg = TokenQueryMsg::OwnerOf {
            token_id: "3".to_string(),
            include_expired: None,
        };
        let res: Result<OwnerOfResponse, cosmwasm_std::StdError> =
            app.wrap().query_wasm_smart(collection_1_addr.clone(), &msg);
        assert!(res.is_err());

        let collection_2_addr = get_collection_address(&mut app, &mint_module_addr.to_string(), 2);

        let msg = TokenQueryMsg::OwnerOf {
            token_id: "1".to_string(),
            include_expired: None,
        };
        let res: OwnerOfResponse = app
            .wrap()
            .query_wasm_smart(collection_2_addr.clone(), &msg)
            .unwrap();
        assert_eq!(res.owner, USER);
    }

    #[test]
    fn test_unhappy_path() {
        let mut app = mock_app();
        let controller_addr = proper_instantiate(&mut app);

        setup_all_modules(&mut app, controller_addr.clone());

        let (mint_module_addr, merge_module_addr, _) =
            get_modules_addresses(&mut app, &controller_addr.to_string());

        let merge_msg = vec![MergeMsg {
            collection_id: 2,
            token_id: None,
            collection_type: Collections::Passcard,
            action: MergeAction::Mint,
        }];
        let msg = MergeExecuteMsg::Merge {
            msg: to_binary(&merge_msg).unwrap(),
        };
        let err = app
            .execute_contract(
                Addr::unchecked(USER),
                merge_module_addr.clone(),
                &msg,
                &vec![],
            )
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            MergeContractError::BurnNotFound {}.to_string()
        );

        let token_contract_code_id = app.store_code(token_contract());
        create_collection(
            &mut app,
            mint_module_addr.clone(),
            token_contract_code_id,
            None,
            None,
            None,
            None,
        );

        let merge_msg = vec![
            MergeMsg {
                collection_id: 1,
                token_id: Some(1),
                collection_type: Collections::Normal,
                action: MergeAction::Burn,
            },
            MergeMsg {
                collection_id: 2,
                token_id: None,
                collection_type: Collections::Passcard,
                action: MergeAction::Mint,
            },
        ];
        let msg = MergeExecuteMsg::Merge {
            msg: to_binary(&merge_msg).unwrap(),
        };
        let err = app
            .execute_contract(Addr::unchecked(USER), merge_module_addr, &msg, &vec![])
            .unwrap_err();
        assert_eq!(
            err.source().unwrap().to_string(),
            MergeContractError::InvalidPasscard {}.to_string()
        );
    }
}
