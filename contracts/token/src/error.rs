use cosmwasm_std::StdError;
use thiserror::Error;

use cw721_base::ContractError as Cw721ContractError;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Minting is locked")]
    MintLocked {},

    #[error("Burning is locked")]
    BurnedLocked {},

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

impl From<ContractError> for Cw721ContractError {
    fn from(err: ContractError) -> Cw721ContractError {
        match err {
            ContractError::Unauthorized {} => Cw721ContractError::Unauthorized {},
            ContractError::Claimed {} => Cw721ContractError::Claimed {},
            ContractError::Expired {} => Cw721ContractError::Expired {},
            _ => unreachable!("cannot convert {:?} to Cw721ContractError", err),
        }
    }
}