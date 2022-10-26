use cosmwasm_std::{Addr, StdError};
use thiserror::Error;

#[cfg(feature = "funds")]
pub mod funds;
#[cfg(feature = "response")]
pub mod response;
#[cfg(feature = "storage")]
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

    if !has_privileges {
        if let Some(parent) = parent_addr {
            if sender == &parent {
                has_privileges = true;
            }
        }
    }

    if !has_privileges {
        if let Some(operators) = operators {
            for operator in operators {
                if sender == &operator {
                    has_privileges = true;
                    break;
                }
            }
        }
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
}
