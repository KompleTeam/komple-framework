use cosmwasm_std::StdError;
use thiserror::Error;

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

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("token_id already claimed")]
    Claimed {},

    #[error("Cannot set approval that is already expired")]
    Expired {},
}

impl From<Cw721ContractError> for ContractError {
    fn from(err: Cw721ContractError) -> ContractError {
        match err {
            Cw721ContractError::Unauthorized {} => ContractError::Unauthorized {},
            Cw721ContractError::Claimed {} => ContractError::Claimed {},
            Cw721ContractError::Expired {} => ContractError::Expired {},
            _ => unreachable!("cannot convert {:?} to Cw721ContractError", err),
        }
    }
}
