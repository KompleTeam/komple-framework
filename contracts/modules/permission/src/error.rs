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
    NoPermissionsForModule {},

    #[error("Invalid reply ID")]
    InvalidReplyID {},

    #[error("Error while instantiating {permission:?} permission")]
    PermissionInstantiateError { permission: String },

    #[error("{0}")]
    Util(#[from] UtilError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}