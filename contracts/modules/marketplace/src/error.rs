use cosmwasm_std::{OverflowError, StdError};
use komple_token_module::ContractError as TokenContractError;
use komple_utils::{funds::FundsError, shared::SharedError, UtilError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Execute locked")]
    ExecuteLocked {},

    #[error("Buy locked")]
    BuyLocked {},

    #[error("Invalid instantiate message")]
    InvalidInstantiateMsg {},

    #[error("Token transfer locked")]
    TransferLocked {},

    #[error("Token send locked")]
    SendLocked {},

    #[error("Token burn locked")]
    BurnLocked {},

    #[error("Token is not listed")]
    NotListed {},

    #[error("Cannot make a self purchase")]
    SelfPurchase {},

    #[error("Fee module address not found")]
    FeeModuleNotFound {},

    #[error("Token already listed")]
    AlreadyListed {},

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("{0}")]
    Util(#[from] UtilError),

    #[error("{0}")]
    Funds(#[from] FundsError),

    #[error("{0}")]
    SharedError(#[from] SharedError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),
}

impl From<TokenContractError> for ContractError {
    fn from(err: TokenContractError) -> ContractError {
        match err {
            TokenContractError::TransferLocked {} => ContractError::TransferLocked {},
            TokenContractError::SendLocked {} => ContractError::SendLocked {},
            TokenContractError::BurnLocked {} => ContractError::BurnLocked {},
            _ => unreachable!(),
        }
    }
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
