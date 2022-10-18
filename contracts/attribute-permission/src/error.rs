use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Metadata not found")]
    MetadataNotFound {},

    #[error("Attribute not found")]
    AttributeNotFound {},

    #[error("Unauthorized")]
    Unauthorized {},
}
