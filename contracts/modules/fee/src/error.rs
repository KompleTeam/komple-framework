use cosmwasm_std::{DivideByZeroError, StdError};
use komple_utils::{funds::FundsError, shared::SharedError, UtilError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Execute locked")]
    ExecuteLocked {},

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

    #[error("No payments found for distribution")]
    NoPaymentsFound {},

    #[error("{0}")]
    DivideByZeroError(#[from] DivideByZeroError),

    #[error("{0}")]
    UtilError(#[from] UtilError),

    #[error("{0}")]
    FundsError(#[from] FundsError),

    #[error("{0}")]
    SharedError(#[from] SharedError),
}
