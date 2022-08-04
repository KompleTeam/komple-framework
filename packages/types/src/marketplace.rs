use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
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
