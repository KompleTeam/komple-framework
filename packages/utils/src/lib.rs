use cosmwasm_std::{Addr, DepsMut, QuerierWrapper, StdError, StdResult};
use cw721::OwnerOfResponse;
use rift_types::{
    module::Modules,
    query::{AddressResponse, ControllerQueryMsg, MintModuleQueryMsg, TokenContractQueryMsg},
};
use thiserror::Error;

pub fn check_admin_privileges(
    sender: &Addr,
    contract_addr: &Addr,
    admin: &Addr,
    parent_addr: Option<Addr>,
    whitelist_addrs: Option<Vec<Addr>>,
) -> Result<(), UtilError> {
    let mut has_privileges = sender == contract_addr;

    if !has_privileges && sender == admin {
        has_privileges = true;
    }

    if !has_privileges && parent_addr.is_some() {
        has_privileges = sender == &parent_addr.unwrap();
    }

    if !has_privileges && whitelist_addrs.is_some() {
        has_privileges = whitelist_addrs.unwrap().contains(sender);
    }

    match has_privileges {
        true => Ok(()),
        false => Err(UtilError::Unauthorized {}),
    }
}

pub fn query_module_address(
    deps: &DepsMut,
    controller_addr: &Addr,
    module: Modules,
) -> StdResult<Addr> {
    let res: AddressResponse = deps
        .querier
        .query_wasm_smart(controller_addr, &ControllerQueryMsg::ModuleAddress(module))
        .unwrap();
    Ok(Addr::unchecked(res.address))
}

pub fn query_collection_address(
    deps: &DepsMut,
    module_address: &Addr,
    collection_id: u32,
) -> StdResult<Addr> {
    let res: AddressResponse = deps
        .querier
        .query_wasm_smart(
            module_address,
            &MintModuleQueryMsg::CollectionAddress(collection_id),
        )
        .unwrap();
    Ok(Addr::unchecked(res.address))
}

pub fn query_linked_collections(
    deps: &DepsMut,
    module_address: &Addr,
    collection_id: u32,
) -> StdResult<Vec<u32>> {
    let res: Vec<u32> = deps
        .querier
        .query_wasm_smart(
            module_address,
            &MintModuleQueryMsg::LinkedCollections { collection_id },
        )
        .unwrap();
    Ok(res)
}

pub fn query_token_owner(
    querier: &QuerierWrapper,
    collection_addr: &Addr,
    token_id: String,
) -> StdResult<Addr> {
    let res: OwnerOfResponse = querier
        .query_wasm_smart(
            collection_addr,
            &TokenContractQueryMsg::OwnerOf {
                token_id,
                include_expired: None,
            },
        )
        .unwrap();
    Ok(Addr::unchecked(res.owner))
}

#[derive(Error, Debug, PartialEq)]
pub enum UtilError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
}
