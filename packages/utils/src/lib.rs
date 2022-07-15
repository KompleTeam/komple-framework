use cosmwasm_std::{Addr, DepsMut, StdError};
use rift_types::{
    module::Modules,
    query::{AddressResponse, ControllerQueryMsg, MintModuleQueryMsg},
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

pub fn get_module_address(
    deps: &DepsMut,
    controller_addr: &Addr,
    module: Modules,
) -> Result<Addr, UtilError> {
    let res: AddressResponse = deps
        .querier
        .query_wasm_smart(controller_addr, &ControllerQueryMsg::ModuleAddress(module))
        .unwrap();
    Ok(Addr::unchecked(res.address))
}

pub fn get_collection_address(
    deps: &DepsMut,
    module_address: &Addr,
    collection_id: u32,
) -> Result<Addr, UtilError> {
    let res: AddressResponse = deps
        .querier
        .query_wasm_smart(
            module_address,
            &MintModuleQueryMsg::CollectionAddress(collection_id),
        )
        .unwrap();
    Ok(Addr::unchecked(res.address))
}

pub fn get_linked_collections(
    deps: &DepsMut,
    module_address: &Addr,
    collection_id: u32,
) -> Result<Vec<u32>, UtilError> {
    let res: Vec<u32> = deps
        .querier
        .query_wasm_smart(
            module_address,
            &MintModuleQueryMsg::LinkedCollections { collection_id },
        )
        .unwrap();
    Ok(res)
}

#[derive(Error, Debug, PartialEq)]
pub enum UtilError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
}

#[cfg(test)]
mod tests {
    fn test_check_admin_privileges() {}
}
