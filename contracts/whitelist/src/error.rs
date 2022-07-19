use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid member limit")]
    InvalidMemberLimit {},

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
}
