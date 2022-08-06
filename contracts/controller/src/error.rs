use cosmwasm_std::StdError;
use komple_utils::UtilError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Description too long")]
    DescriptionTooLong {},

    #[error("Invalid reply ID")]
    InvalidReplyID {},

    #[error("Error while instantiating {module:?} module")]
    ModuleInstantiateError { module: String },

    #[error("{0}")]
    Util(#[from] UtilError),
}
