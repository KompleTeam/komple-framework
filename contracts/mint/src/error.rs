use cosmwasm_std::StdError;
use komple_utils::UtilError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Minting locked")]
    LockedMint {},

    #[error("Invalid reply ID")]
    InvalidReplyID {},

    #[error("Error while instantiating token contract")]
    TokenInstantiateError {},

    #[error("Invalid collection ID")]
    InvalidCollectionId {},

    #[error("Invalid metadata IDs")]
    InvalidMetadataIds {},

    #[error("Collection cannot be linked to itself")]
    SelfLinkedCollection {},

    #[error("{0}")]
    Util(#[from] UtilError),
}
