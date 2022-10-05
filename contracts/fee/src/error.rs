use cosmwasm_std::{DivideByZeroError, StdError};
use komple_utils::funds::FundsError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid fee")]
    InvalidFee {},

    #[error("Fee already exists")]
    ExistingFee {},

    #[error("Share already exists")]
    ExistingShare {},

    #[error("Fee not found")]
    FeeNotFound {},

    #[error("Total fee cannot exceed 1")]
    InvalidTotalFee {},

    #[error("Invalid funds")]
    InvalidFunds {},

    #[error("No payments found for distribution")]
    NoPaymentsFound {},

    #[error("{0}")]
    DivideByZeroError(#[from] DivideByZeroError),

    #[error("{0}")]
    FundsError(#[from] FundsError),
}
