use cosmwasm_std::StdError;
use komple_utils::UtilError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid permissions")]
    InvalidPermissions {},

    #[error("Invalid ownership")]
    InvalidOwnership {},

    #[error("No permissions found for module")]
    NoPermissionsInModule {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("{0}")]
    Util(#[from] UtilError),
}
