use cosmwasm_std::StdError;
use rift_utils::UtilError;
use thiserror::Error;
use url::ParseError;

use cw721_base::ContractError as Cw721ContractError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Minting is locked")]
    MintLocked {},

    #[error("Burning is locked")]
    BurnLocked {},

    #[error("Transferring is locked")]
    TransferLocked {},

    #[error("Sending is locked")]
    SendLocked {},

    #[error("Per address limit must be greater than 0")]
    InvalidPerAddressLimit {},

    #[error("Token limit is exceeded")]
    TokenLimitReached {},

    #[error("Invalid start time")]
    InvalidStartTime {},

    #[error("Minting has already started")]
    AlreadyStarted {},

    #[error("Description too long")]
    DescriptionTooLong {},

    #[error("token_id already claimed")]
    Claimed {},

    #[error("Cannot set approval that is already expired")]
    Expired {},

    #[error("Approval not found for: {spender}")]
    ApprovalNotFound { spender: String },

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("{0}")]
    Parse(#[from] ParseError),

    #[error("{0}")]
    Util(#[from] UtilError),
}

impl From<Cw721ContractError> for ContractError {
    fn from(err: Cw721ContractError) -> ContractError {
        match err {
            Cw721ContractError::Std(err) => ContractError::Std(err),
            Cw721ContractError::Unauthorized {} => ContractError::Unauthorized {},
            Cw721ContractError::Claimed {} => ContractError::Claimed {},
            Cw721ContractError::Expired {} => ContractError::Expired {},
            Cw721ContractError::ApprovalNotFound { spender } => {
                ContractError::ApprovalNotFound { spender }
            }
        }
    }
}
