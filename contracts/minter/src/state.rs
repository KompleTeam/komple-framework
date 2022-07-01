use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionInfo {
    pub name: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
}
pub const COLLECTION_INFO: Item<CollectionInfo> = Item::new("collection_info");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RoyaltyInfo {
    pub payment_addr: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub royalty_info: Option<RoyaltyInfo>,
}
pub const CONFIG: Item<Config> = Item::new("config");

pub const TOKEN_ID: Item<u32> = Item::new("token_id");

pub const MINTERS: Map<Addr, u32> = Map::new("minters");

pub const TOKEN_ADDR: Item<Addr> = Item::new("token_addr");
