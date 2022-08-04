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
}

impl From<Cw721ContractError> for ContractError {
    fn from(err: Cw721ContractError) -> ContractError {
        match err {
            _ => unreachable!("cannot convert {:?} to Cw721ContractError", err),
        }
    }
}
