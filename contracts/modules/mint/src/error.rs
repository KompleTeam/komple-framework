use cosmwasm_std::StdError;
use komple_utils::{funds::FundsError, UtilError};
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

    #[error("Collection ID not found")]
    CollectionIdNotFound {},

    #[error("Collection cannot be linked to itself")]
    SelfLinkedCollection {},

    #[error("Collection is already blacklisted")]
    AlreadyBlacklisted {},

    #[error("Collection is already whitelistlisted")]
    AlreadyWhitelistlisted {},

    #[error("Address is not whitelisted")]
    AddressNotWhitelisted {},

    #[error("Whitelist price is not set")]
    WhitelistPriceNotSet {},

    #[error("{0}")]
    Util(#[from] UtilError),

    #[error("{0}")]
    Funds(#[from] FundsError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
