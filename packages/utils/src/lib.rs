use cosmwasm_std::{Addr, DepsMut, StdError};
use rift_types::{
    module::Modules,
    query::{AddressResponse, ControllerQueryMsg, MintModuleQueryMsg},
};
use thiserror::Error;

pub fn check_admin_privilages(
    sender: &Addr,
    admin: &Addr,
    parent_contract: Option<&Addr>,
    enabled_modules: Option<&Vec<Addr>>,
) -> Result<(), UtilError> {
    if admin != sender {
        return Err(UtilError::Unauthorized {});
    }
    if parent_contract.is_some() && parent_contract.unwrap() != sender {
        return Err(UtilError::Unauthorized {});
    }
    if enabled_modules.is_some() && !enabled_modules.unwrap().contains(&sender) {
        return Err(UtilError::Unauthorized {});
    }
    Ok(())
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

#[derive(Error, Debug, PartialEq)]
pub enum UtilError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
}
