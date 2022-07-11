use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid reply ID")]
    InvalidReplyID {},

    #[error("Error while instantiating token contract")]
    TokenInstantiateError {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
}
