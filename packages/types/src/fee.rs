use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Uint128};

/// The different types of fees.
///
/// Currently only percentage and fixed fees are supported.
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

/// The different type of mint fees to be used in mint module.
///
/// This is used for convinience when setting the fee configuration.
#[cw_serde]
pub enum MintFees {
    Price,
    Whitelist,
    Royalty,
}
impl MintFees {
    pub fn as_str(&self) -> &'static str {
        match self {
            MintFees::Price => "price",
            MintFees::Whitelist => "whitelist",
            MintFees::Royalty => "royalty",
        }
    }
    pub fn new_price(collection_id: u32) -> String {
        format!("{}:{}", MintFees::Price.as_str(), collection_id)
    }
    pub fn new_whitelist_price(collection_id: u32) -> String {
        format!("{}:{}", MintFees::Whitelist.as_str(), collection_id)
    }
    pub fn new_royalty(collection_id: u32) -> String {
        format!("{}:{}", MintFees::Royalty.as_str(), collection_id)
    }
}

/// The different type of marketplace fees to be used in marketplace module.
///
/// This is used for convinience when setting the fee configuration.
#[cw_serde]
pub enum MarketplaceFees {
    Komple,
    Community,
    HubAdmin,
}
impl MarketplaceFees {
    pub fn as_str(&self) -> &'static str {
        match self {
            MarketplaceFees::Komple => "komple",
            MarketplaceFees::Community => "community",
            MarketplaceFees::HubAdmin => "hub_admin",
        }
    }
}

/// The payment configuration for a percentage fee.
///
/// This is saved to storage for a module and fee name.
#[cw_serde]
pub struct PercentagePayment {
    /// Address is the payment address.
    /// If the address is empty, custom payment addresses are used for distribution.
    pub address: Option<String>,
    /// Value is the percentage value.
    pub value: Decimal,
}

/// The payment configuration for a fixed fee.
///
/// This is saved to storage for a module and fee name.
#[cw_serde]
pub struct FixedPayment {
    /// Address is the payment address.
    /// If the address is empty, custom payment addresses are used for distribution.
    pub address: Option<String>,
    /// Value is the integer value.
    pub value: Uint128,
}

pub const FIXED_FEES_NAMESPACE: &str = "fixed_fees";

pub const PERCENTAGE_FEES_NAMESPACE: &str = "percentage_fees";
