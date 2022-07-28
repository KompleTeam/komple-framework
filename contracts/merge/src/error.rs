use cosmwasm_std::StdError;
use rift_utils::UtilError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("No burn messages found")]
    BurnNotFound {},

    #[error("No linked collections found in burn message")]
    LinkedCollectionNotFound {},

    #[error("Invalid metadata IDs")]
    InvalidMetadataIds {},

    #[error("{0}")]
    Util(#[from] UtilError),
}
