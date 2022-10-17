use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Uint128};

#[cw_serde]
pub enum Fees {
    Fixed,
    Percentage,
}

impl Fees {
    pub fn as_str(&self) -> &'static str {
        match self {
            Fees::Fixed => "fixed",
            Fees::Percentage => "percentage",
        }
    }
}

#[cw_serde]
pub struct PercentagePayment {
    // Address is optional and can be overriden with a custom address on distribution
    pub address: Option<String>,
    pub value: Decimal,
}

#[cw_serde]
pub struct FixedPayment {
    // Address is optional and can be overriden with a custom address on distribution
    pub address: Option<String>,
    pub value: Uint128,
}

pub const FIXED_FEES_NAMESPACE: &str = "fixed_fees";

pub const PERCENTAGE_FEES_NAMESPACE: &str = "percentage_fees";
