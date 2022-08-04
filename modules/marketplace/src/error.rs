use cosmwasm_std::{OverflowError, StdError};
use rift_utils::UtilError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid funds")]
    InvalidFunds {},

    #[error("Invalid denom")]
    InvalidDenom {},

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("{0}")]
    Util(#[from] UtilError),
}
