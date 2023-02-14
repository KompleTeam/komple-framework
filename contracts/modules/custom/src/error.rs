use cosmwasm_std::StdError;
use komple_framework_utils::shared::SharedError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    /* TODO: Add custom errors here */
    /* ... */
    #[error("Execute locked")]
    ExecuteLocked {},

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
