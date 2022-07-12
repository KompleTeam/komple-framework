use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MINT_MODULE_INSTANTIATE_REPLY_ID: u64 = 1;
pub const PASSCARD_MODULE_INSTANTIATE_REPLY_ID: u64 = 2;
pub const PERMISSION_MODULE_INSTANTIATE_REPLY_ID: u64 = 3;
pub const SWAP_MODULE_INSTANTIATE_REPLY_ID: u64 = 4;
pub const MERGE_MODULE_INSTANTIATE_REPLY_ID: u64 = 5;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Modules {
    MintModule,
    PasscardModule,
    PermissionModule,
    SwapModule,
    MergeModule,
}

impl Modules {
    pub fn to_string(&self) -> &str {
        match self {
            Modules::MintModule => "mint",
            Modules::PasscardModule => "passcard",
            Modules::PermissionModule => "permission",
            Modules::SwapModule => "swap",
            Modules::MergeModule => "merge",
        }
    }
}
