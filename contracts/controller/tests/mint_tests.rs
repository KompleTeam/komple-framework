use controller_contract::msg::ExecuteMsg;
use cosmwasm_std::Addr;
use cw_multi_test::Executor;

pub mod helpers;
use helpers::{
    create_collection, get_modules_addresses, mint_module, mint_token, mock_app,
    proper_instantiate, setup_all_modules, setup_metadata, setup_metadata_contract, token_contract,
    ADMIN, USER,
};

mod initialization {
    use super::*;

    use komple_types::module::Modules;

    use controller_contract::ContractError;
    use komple_utils::query_module_address;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let controller_addr = proper_instantiate(&mut app);
        let mint_module_code_id = app.store_code(mint_module());

        let msg = ExecuteMsg::InitMintModule {
            code_id: mint_module_code_id,
        };
        let _ = app.execute_contract(
            Addr::unchecked(ADMIN),
            controller_addr.clone(),
            &msg,
            &vec![],
        );

        let res = query_module_address(&app.wrap(), &controller_addr, Modules::MintModule).unwrap();
        assert_eq!(res, "contract1")
    }

    #[test]
    fn test_invalid_sender() {
        let mut app = mock_app();
        let controller_addr = proper_instantiate(&mut app);
        let mint_module_code_id = app.store_code(mint_module());

        let msg = ExecuteMsg::InitMergeModule {
            code_id: mint_module_code_id,
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

mod permission_mint {
    use helpers::add_permission_for_module;
    use komple_utils::query_collection_address;

    use super::*;

    use cosmwasm_std::to_binary;
    use cw721::OwnerOfResponse;
    use mint_module::msg::ExecuteMsg as MintExecuteMsg;
    use permission_module::msg::{OwnershipMsg, PermissionCheckMsg};
    use komple_types::{
        collection::Collections, metadata::Metadata, module::Modules, permission::Permissions,
    };
    use token_contract::msg::QueryMsg as TokenQueryMsg;

    #[test]
    fn test_happy_path() {
        let mut app = mock_app();
        let controller_addr = proper_instantiate(&mut app);

        setup_all_modules(&mut app, controller_addr.clone());

        let (mint_module_addr, _, permission_module_addr, _) =
            get_modules_addresses(&mut app, &controller_addr);

        let token_contract_code_id = app.store_code(token_contract());
        create_collection(
            &mut app,
            mint_module_addr.clone(),
            token_contract_code_id,
            None,
            None,
            Collections::Normal,
            None,
            None,
        );
        create_collection(
            &mut app,
            mint_module_addr.clone(),
            token_contract_code_id,
            None,
            None,
            Collections::Normal,
            None,
            None,
        );

        let collection_addr_1 =
            query_collection_address(&app.wrap(), &mint_module_addr.clone(), &1).unwrap();
        let collection_addr_2 =
            query_collection_address(&app.wrap(), &mint_module_addr.clone(), &2).unwrap();

        let metadata_contract_addr_1 =
            setup_metadata_contract(&mut app, collection_addr_1, Metadata::OneToOne);
        let metadata_contract_addr_2 =
            setup_metadata_contract(&mut app, collection_addr_2, Metadata::OneToOne);

        setup_metadata(&mut app, metadata_contract_addr_1.clone());
        setup_metadata(&mut app, metadata_contract_addr_1.clone());
        setup_metadata(&mut app, metadata_contract_addr_2);

        mint_token(&mut app, mint_module_addr.clone(), 1, USER);
        mint_token(&mut app, mint_module_addr.clone(), 1, USER);

        add_permission_for_module(
            &mut app,
            permission_module_addr,
            Modules::MintModule,
            vec![Permissions::Ownership],
        );

        let permission_msg = to_binary(&vec![PermissionCheckMsg {
            permission_type: Permissions::Ownership,
            data: to_binary(&vec![
                OwnershipMsg {
                    collection_id: 1,
                    token_id: 1,
                    owner: USER.to_string(),
                },
                OwnershipMsg {
                    collection_id: 1,
                    token_id: 2,
                    owner: USER.to_string(),
                },
            ])
            .unwrap(),
        }])
        .unwrap();
        let collection_ids = vec![2];
        let msg = MintExecuteMsg::PermissionMint {
            permission_msg,
            collection_ids,
            metadata_ids: None,
        };
        let _ = app
            .execute_contract(
                Addr::unchecked(USER),
                mint_module_addr.clone(),
                &msg,
                &vec![],
            )
            .unwrap();

        let collection_2_addr =
            query_collection_address(&app.wrap(), &mint_module_addr, &2).unwrap();

        let msg = TokenQueryMsg::OwnerOf {
            token_id: "1".to_string(),
            include_expired: None,
        };
        let res: OwnerOfResponse = app
            .wrap()
            .query_wasm_smart(collection_2_addr, &msg)
            .unwrap();
        assert_eq!(res.owner, USER);
    }
}
