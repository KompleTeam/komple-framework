use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Locks {
    pub burn_lock: bool,
    pub mint_lock: bool,
    pub transfer_lock: bool,
    pub send_lock: bool,
}

pub const TOKENS_NAMESPACE: &str = "tokens";

pub const TOKEN_IDS_NAMESPACE: &str = "token_ids";

pub const TOKEN_LOCKS_NAMESPACE: &str = "token_locks";

pub const LOCKS_NAMESPACE: &str = "locks";

pub const CONTRACTS_NAMESPACE: &str = "contracts";

pub const BUNDLE_CONFIG_NAMESPACE: &str = "bundle_config";

pub const MINT_MODULE_ADDR_NAMESPACE: &str = "mint_module_addr";

pub const MINTED_TOKENS_PER_ADDR_NAMESPACE: &str = "minted_tokens_per_addr";

pub const BUNDLE_INFO_NAMESPACE: &str = "bundle_info";
