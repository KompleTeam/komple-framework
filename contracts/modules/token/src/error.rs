use cosmwasm_std::StdError;
use komple_utils::UtilError;
use thiserror::Error;

use cw721_base::ContractError as Cw721ContractError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid instantiate msg")]
    InvalidInstantiateMsg {},

    #[error("Token mint locked")]
    MintLocked {},

    #[error("Token burn locked")]
    BurnLocked {},

    #[error("Token transfer locked")]
    TransferLocked {},

    #[error("Token send locked")]
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

    #[error("Token_id already claimed")]
    Claimed {},

    #[error("Cannot set approval that is already expired")]
    Expired {},

    #[error("Approval not found for: {spender}")]
    ApprovalNotFound { spender: String },

    #[error("Invalid reply ID")]
    InvalidReplyID {},

    #[error("Metadata contract not found")]
    MetadataContractNotFound {},

    #[error("Error while instantiating contract")]
    ContractsInstantiateError {},

    #[error("IPFS link not found")]
    IpfsNotFound {},

    #[error("Collection and metadata types must be standard")]
    InvalidCollectionMetadataType {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("{0}")]
    Util(#[from] UtilError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),
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

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
