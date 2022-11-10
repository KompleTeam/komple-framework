use cosmwasm_schema::cw_serde;
use std::fmt;

/// The different types of listing.
///
/// Currently only fixed and auction listings are supported.
#[cw_serde]
pub enum Listing {
    Fixed,
    Auction,
}

impl Listing {
    pub fn as_str(&self) -> &'static str {
        match self {
            Listing::Fixed => "fixed",
            Listing::Auction => "auction",
        }
    }
}

impl fmt::Display for Listing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Listing::Fixed => write!(f, "fixed"),
            Listing::Auction => write!(f, "auction"),
        }
    }
}

pub const FIXED_LISTING_NAMESPACE: &str = "fixed_listing";
