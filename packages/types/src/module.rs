use cosmwasm_schema::cw_serde;
use std::fmt;

#[cw_serde]
pub enum Modules {
    Hub,
    Mint,
    Permission,
    Swap,
    Merge,
    Marketplace,
    Fee,
}

impl Modules {
    pub fn as_str(&self) -> &str {
        match self {
            Modules::Hub => "hub",
            Modules::Mint => "mint",
            Modules::Permission => "permission",
            Modules::Swap => "swap",
            Modules::Merge => "merge",
            Modules::Marketplace => "marketplace",
            Modules::Fee => "fee",
        }
    }
}

impl fmt::Display for Modules {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Modules::Hub => write!(f, "hub"),
            Modules::Mint => write!(f, "mint"),
            Modules::Permission => write!(f, "permission"),
            Modules::Swap => write!(f, "swap"),
            Modules::Merge => write!(f, "merge"),
            Modules::Marketplace => write!(f, "marketplace"),
            Modules::Fee => write!(f, "fee"),
        }
    }
}

pub const MODULES_NAMESPACE: &str = "modules";
