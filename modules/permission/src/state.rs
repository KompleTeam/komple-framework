use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
}
pub const CONFIG: Item<Config> = Item::new("config");

pub const PASSCARD_ADDR: Map<u32, Addr> = Map::new("passcard_addr");

pub const PASSCARD_ID: Item<u32> = Item::new("passcard_id");

pub const CONTROLLER_ADDR: Item<Addr> = Item::new("controller_addr");

// passcard_id -> vec<collection_id>
pub const MAIN_COLLECTIONS: Map<u32, Vec<u32>> = Map::new("main_collections");
