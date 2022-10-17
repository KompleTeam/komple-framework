use cosmwasm_std::{Coin, MessageInfo, StdError, Uint128};
use thiserror::Error;

pub fn check_single_amount(info: &MessageInfo, amount: Uint128) -> Result<(), FundsError> {
    if info.funds.len() != 1 {
        return Err(FundsError::MissingFunds {});
    };
    let sent_fund = info.funds.get(0).unwrap();
    if sent_fund.amount != amount {
        return Err(FundsError::InvalidFunds {
            got: sent_fund.amount.to_string(),
            expected: amount.to_string(),
        });
    }
    Ok(())
}

pub fn check_single_coin(info: &MessageInfo, expected: Coin) -> Result<(), FundsError> {
    if info.funds.len() != 1 {
        return Err(FundsError::MissingFunds {});
    };
    let sent_fund = info.funds.get(0).unwrap();
    if sent_fund.denom != expected.denom {
        return Err(FundsError::InvalidDenom {
            got: sent_fund.denom.to_string(),
            expected: expected.denom,
        });
    }
    if sent_fund.amount != expected.amount {
        return Err(FundsError::InvalidFunds {
            got: sent_fund.amount.to_string(),
            expected: expected.amount.to_string(),
        });
    }
    Ok(())
}

#[derive(Error, Debug, PartialEq)]
pub enum FundsError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Invalid denom! Got: {got} - Expected: {expected}")]
    InvalidDenom { got: String, expected: String },

    #[error("Invalid funds! Got: {got} - Expected: {expected}")]
    InvalidFunds { got: String, expected: String },

    #[error("No funds found!")]
    MissingFunds {},
}
