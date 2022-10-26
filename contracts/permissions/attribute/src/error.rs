use cosmwasm_std::StdError;
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Metadata not found")]
    MetadataNotFound {},

    #[error("Attribute not found")]
    AttributeNotFound {},

    #[error("Attribute found")]
    AttributeFound {},

    #[error("Attribute not equal")]
    AttributeNotEqual {},

    #[error("Attribute equal")]
    AttributeEqual {},

    #[error("Attribute is greater than")]
    AttributeGreaterThan {},

    #[error("Attribute is greater than or equal")]
    AttributeGreaterThanOrEqual {},

    #[error("Attribute is less than")]
    AttributeLessThan {},

    #[error("Attribute is less than or equal")]
    AttributeLessThanOrEqual {},

    #[error("Attribute type mismatch")]
    AttributeTypeMismatch {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("{0}")]
    ParseInt(#[from] ParseIntError),
}
