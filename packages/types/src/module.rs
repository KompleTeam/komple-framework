use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MINT_MODULE_INSTANTIATE_REPLY_ID: u64 = 1;
pub const PERMISSION_MODULE_INSTANTIATE_REPLY_ID: u64 = 2;
pub const SWAP_MODULE_INSTANTIATE_REPLY_ID: u64 = 3;
pub const MERGE_MODULE_INSTANTIATE_REPLY_ID: u64 = 4;
pub const MARKETPLACE_MODULE_INSTANTIATE_REPLY_ID: u64 = 5;

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
}

pub const MODULE_ADDRS_NAMESPACE: &str = "module_addrs";
