use cosmwasm_std::{DivideByZeroError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid percentage")]
    InvalidPercentage {},

    #[error("Share already exists")]
    ExistingShare {},

    #[error("Share not found")]
    ShareNotFound {},

    #[error("Total fee cannot exceed 1")]
    InvalidTotalFee {},

    #[error("{0}")]
    DivideByZeroError(#[from] DivideByZeroError),
}
