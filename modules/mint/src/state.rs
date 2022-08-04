use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub mint_lock: bool,
}
pub const CONFIG: Item<Config> = Item::new("config");

pub const TOKEN_ADDRS: Map<u32, Addr> = Map::new("token_addrs");

pub const COLLECTION_ID: Item<u32> = Item::new("collection_id");

pub const CONTROLLER_ADDR: Item<Addr> = Item::new("controller_addr");
