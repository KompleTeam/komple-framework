use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid instantiate msg")]
    InvalidInstantiateMsg {},

    #[error("Invalid member limit")]
    InvalidMemberLimit {},

    #[error("Member list cannot be empty")]
    EmptyMemberList {},

    #[error("Member limit exceeded")]
    MemberLimitExceeded {},

    #[error("Invalid per address limit")]
    InvalidPerAddressLimit {},

    #[error("Minting has already started")]
    AlreadyStarted {},

    #[error("Invalid start time")]
    InvalidStartTime {},

    #[error("Invalid end time")]
    InvalidEndTime {},

    #[error("Member already exists")]
    MemberExists {},

    #[error("Member not found")]
    MemberNotFound {},

    #[error("Semver parsing error: {0}")]
    SemVer(String),
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
