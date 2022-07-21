use cosmwasm_std::StdError;
use rift_utils::UtilError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid fee percentage")]
    InvalidFeePercentage {},

    #[error("{0}")]
    Util(#[from] UtilError),
}
