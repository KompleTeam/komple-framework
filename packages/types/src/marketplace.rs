use cosmwasm_schema::cw_serde;

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
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

pub const FIXED_LISTING_NAMESPACE: &str = "fixed_listing";
