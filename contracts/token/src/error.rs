use cosmwasm_std::StdError;
use rift_utils::{FundsError, UtilError};
use thiserror::Error;

use cw721_base::ContractError as Cw721ContractError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Token mint locked")]
    MintLocked {},

    #[error("Token burn locked")]
    BurnLocked {},

    #[error("Token transfer locked")]
    TransferLocked {},

    #[error("token send locked")]
    SendLocked {},

    #[error("Per address limit must be greater than 0")]
    InvalidPerAddressLimit {},

    #[error("Token limit is exceeded")]
    TokenLimitReached {},

    #[error("Token not found")]
    TokenNotFound {},

    #[error("Invalid max token limit")]
    InvalidMaxTokenLimit {},

    #[error("Invalid start time")]
    InvalidStartTime {},

    #[error("Minting has not started")]
    MintingNotStarted {},

    #[error("Minting has already started")]
    AlreadyStarted {},

    #[error("Description too long")]
    DescriptionTooLong {},

    #[error("Address is not whitelisted")]
    NotWhitelisted {},

    #[error("token_id already claimed")]
    Claimed {},

    #[error("Cannot set approval that is already expired")]
    Expired {},

    #[error("Approval not found for: {spender}")]
    ApprovalNotFound { spender: String },

    #[error("Invalid reply ID")]
    InvalidReplyID {},

    #[error("Error while instantiating metadata contract")]
    MetadataInstantiateError {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("{0}")]
    Util(#[from] UtilError),

    #[error("{0}")]
    Funds(#[from] FundsError),
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
