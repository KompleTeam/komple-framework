use cosmwasm_std::{Uint128, MessageInfo, Coin, StdError};
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

pub fn check_single_coin(info: &MessageInfo, coin: Coin) -> Result<(), FundsError> {
    if info.funds.len() != 1 {
        return Err(FundsError::MissingFunds {});
    };
    let sent_fund = info.funds.get(0).unwrap();
    if sent_fund.denom != coin.denom {
        return Err(FundsError::InvalidDenom {
            got: sent_fund.denom.to_string(),
            expected: coin.denom.to_string(),
        });
    }
    if sent_fund.amount != coin.amount {
        return Err(FundsError::InvalidFunds {
            got: sent_fund.amount.to_string(),
            expected: coin.amount.to_string(),
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

