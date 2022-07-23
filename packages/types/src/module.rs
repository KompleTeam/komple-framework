use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MINT_MODULE_INSTANTIATE_REPLY_ID: u64 = 1;
pub const PERMISSION_MODULE_INSTANTIATE_REPLY_ID: u64 = 2;
pub const SWAP_MODULE_INSTANTIATE_REPLY_ID: u64 = 3;
pub const MERGE_MODULE_INSTANTIATE_REPLY_ID: u64 = 4;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Modules {
    MintModule,
    PermissionModule,
    SwapModule,
    MergeModule,
    MarketplaceModule,
}

impl Modules {
    pub fn as_str(&self) -> &str {
        match self {
            Modules::MintModule => "mint",
            Modules::PermissionModule => "permission",
            Modules::SwapModule => "swap",
            Modules::MergeModule => "merge",
            Modules::MarketplaceModule => "marketplace",
        }
    }
}

pub const MODULE_ADDRS_NAMESPACE: &str = "module_addrs";
