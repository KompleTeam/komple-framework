use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ControllerInfo {
    pub name: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
}
pub const CONTROLLER_INFO: Item<ControllerInfo> = Item::new("controller_info");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub mint_code_id: u64,
}
pub const CONFIG: Item<Config> = Item::new("config");

pub const COLLECTION_ID: Item<u32> = Item::new("collection_id");

pub const COLLECTIONS: Map<u32, Addr> = Map::new("collections");
