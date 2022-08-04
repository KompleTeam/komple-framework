use rift_types::royalty::Royalty;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub share: Decimal,
    pub royalty_type: Royalty,
}
pub const CONFIG: Item<Config> = Item::new("config");

// Parent collection address
pub const COLLECTION_ADDR: Item<Addr> = Item::new("collection_addr");

// (owner, delegated address)
// If does not exist, it means the owner is the delegated address
pub const OWNER_ROYALTY_ADDR: Map<Addr, Addr> = Map::new("owner_royalty_addr");

// (collection_id, token_id) -> delegated address
// If does not exist, it means the owner is the delegated address
pub const TOKEN_ROYALTY_ADDR: Map<(u32, u32), Addr> = Map::new("token_royalty_addr");
