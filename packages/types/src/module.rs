use cosmwasm_schema::cw_serde;

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
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

pub const MODULE_ADDRS_NAMESPACE: &str = "module_addrs";
