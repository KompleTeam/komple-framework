use cosmwasm_std::{Addr, StdError};
use std::str::Utf8Error;
use thiserror::Error;

pub mod funds;
pub mod storage;

pub fn check_admin_privileges(
    sender: &Addr,
    contract_addr: &Addr,
    admin: &Addr,
    parent_addr: Option<Addr>,
    operators: Option<Vec<Addr>>,
) -> Result<(), UtilError> {
    let mut has_privileges = sender == contract_addr;

    if !has_privileges && sender == admin {
        has_privileges = true;
    }

    if !has_privileges && parent_addr.is_some() {
        has_privileges = sender == &parent_addr.unwrap();
    }

    if !has_privileges && operators.is_some() {
        has_privileges = operators.unwrap().contains(sender);
    }

    match has_privileges {
        true => Ok(()),
        false => Err(UtilError::Unauthorized {}),
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
