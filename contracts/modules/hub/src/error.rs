use cosmwasm_std::StdError;
use komple_framework_utils::{funds::FundsError, shared::SharedError, UtilError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid instantiate message")]
    InvalidInstantiateMsg {},

    #[error("Description too long")]
    DescriptionTooLong {},

    #[error("Invalid reply ID")]
    InvalidReplyID {},

    #[error("Error while instantiating {module:?} module")]
    ModuleInstantiateError { module: String },

    #[error("Module is invalid")]
    InvalidModule {},

    #[error("{0}")]
    Util(#[from] UtilError),

    #[error("{0}")]
    Funds(#[from] FundsError),

    #[error("{0}")]
    SharedError(#[from] SharedError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
