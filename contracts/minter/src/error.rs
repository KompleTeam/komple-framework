use cosmwasm_std::StdError;
use thiserror::Error;
use url::ParseError;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Description too long")]
    DescriptionTooLong {},

    #[error("Invalid start time")]
    InvalidStartTime {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("{0}")]
    Parse(#[from] ParseError),
}
