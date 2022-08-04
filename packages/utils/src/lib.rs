use std::{
    ops::Deref,
    str::{from_utf8, Utf8Error},
};

use cosmwasm_std::{Addr, DepsMut, QuerierWrapper, StdError, StdResult};
use cw721::OwnerOfResponse;
use cw_storage_plus::Path;
use rift_types::{
    module::Modules,
    query::{AddressResponse, ControllerQueryMsg, MintModuleQueryMsg, TokenContractQueryMsg},
};
use schemars::_serde_json::from_str;
use serde::de::DeserializeOwned;
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
    querier: &QuerierWrapper,
    controller_addr: &Addr,
    module: Modules,
) -> StdResult<Addr> {
    let key = get_map_storage_key("module_address", module.as_str().as_bytes())?;
    let res = query_storage::<Addr>(&querier, &controller_addr, &key)?;
    // TODO: Handle none value?
    Ok(res.unwrap())
}

pub fn query_collection_address(
    querier: &QuerierWrapper,
    mint_module_address: &Addr,
    collection_id: u32,
) -> StdResult<Addr> {
    let key = get_map_storage_key("collection_addrs", &collection_id.to_be_bytes())?;
    let res = query_storage::<Addr>(&querier, &mint_module_address, &key)?;
    // TODO: Handle none value?
    Ok(res.unwrap())
}

pub fn query_linked_collections(
    querier: &QuerierWrapper,
    mint_module_address: &Addr,
    collection_id: u32,
) -> StdResult<Vec<u32>> {
    let key = get_map_storage_key("linked_collections", &collection_id.to_be_bytes())?;
    let res = query_storage::<Vec<u32>>(&querier, &mint_module_address, &key)?;
    match res {
        Some(v) => Ok(v),
        None => Ok(vec![]),
    }
}

pub fn query_token_owner(
    querier: &QuerierWrapper,
    collection_addr: &Addr,
    token_id: String,
) -> StdResult<Addr> {
    let key = get_map_storage_key("tokens", token_id.as_bytes())?;
    let res = query_storage::<OwnerOfResponse>(&querier, &collection_addr, &key)?;
    // TODO: Handle none value?
    Ok(Addr::unchecked(res.unwrap().owner))
}

// namespace -> storage key
// key_name -> item key
pub fn get_map_storage_key(namepspace: &str, key_bytes: &[u8]) -> StdResult<String> {
    let namespace_bytes = namepspace.as_bytes();
    let path: Path<Vec<u32>> = Path::new(namespace_bytes, &[key_bytes]);
    let path_str = from_utf8(path.deref())?;
    Ok(path_str.to_string())
}

// To find the key value in storage, we need to construct a path to the key
// For Map storage this key is generated with get_map_storage_key
// For Item storage this key is the namespace value
pub fn query_storage<T>(querier: &QuerierWrapper, addr: &Addr, key: &str) -> StdResult<Option<T>>
where
    T: DeserializeOwned,
{
    let data = querier.query_wasm_raw(addr, key.as_bytes())?;
    match data {
        Some(data) => {
            let res = from_utf8(&data)?;
            let res = from_str(&res).unwrap();
            Ok(Some(res))
        }
        None => Ok(None),
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum UtilError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("{0}")]
    Utf8(#[from] Utf8Error),
}
