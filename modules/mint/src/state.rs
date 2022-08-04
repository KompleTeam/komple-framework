use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub mint_lock: bool,
}
pub const CONFIG: Item<Config> = Item::new("config");

// Wrapped cw721-base contract address
pub const TOKEN_ADDR: Item<Addr> = Item::new("token_addr");

pub const COLLECTION_ID: Item<u32> = Item::new("collection_id");
