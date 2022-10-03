use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Modules {
    Mint,
    Permission,
    Swap,
    Merge,
    Marketplace,
}

impl Modules {
    pub fn as_str(&self) -> &str {
        match self {
            Modules::Mint => "mint",
            Modules::Permission => "permission",
            Modules::Swap => "swap",
            Modules::Merge => "merge",
            Modules::Marketplace => "marketplace",
        }
    }
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

pub const MODULE_ADDRS_NAMESPACE: &str = "module_addrs";
