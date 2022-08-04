use cosmwasm_std::{OverflowError, StdError};
use rift_utils::UtilError;
use thiserror::Error;
use token_contract::ContractError as TokenContractError;

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

    #[error("Token transfer locked")]
    TransferLocked {},

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("{0}")]
    Util(#[from] UtilError),
}

impl From<TokenContractError> for ContractError {
    fn from(err: TokenContractError) -> ContractError {
        match err {
            TokenContractError::TransferLocked {} => ContractError::TransferLocked {},
            _ => unreachable!(),
        }
    }
}
