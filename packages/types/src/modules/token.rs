use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct Locks {
    pub burn_lock: bool,
    pub mint_lock: bool,
    pub transfer_lock: bool,
    pub send_lock: bool,
}

#[cw_serde]
pub struct SubModules {
    pub metadata: Option<Addr>,
    pub whitelist: Option<Addr>,
}

pub const TOKENS_NAMESPACE: &str = "tokens";

pub const TOKEN_IDS_NAMESPACE: &str = "token_ids";

pub const TOKEN_LOCKS_NAMESPACE: &str = "token_locks";

pub const LOCKS_NAMESPACE: &str = "locks";

pub const SUB_MODULES_NAMESPACE: &str = "sub_modules";

pub const MINTED_TOKENS_PER_ADDR_NAMESPACE: &str = "minted_tokens_per_addr";

pub const COLLECTION_TYPE_NAMESPACE: &str = "collection_type";
