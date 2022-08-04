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

pub const TOKEN_LOCKS_NAMESPACE: &str = "token_locks";

pub const LOCKS_NAMESPACE: &str = "locks";

pub const OPERATION_LOCK_NAMESPACE: &str = "operation_lock";

pub const CONTRACTS_NAMESPACE: &str = "contracts";
