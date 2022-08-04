use rift_types::collection::Collections;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionInfo {
    pub collection_type: Collections,
    pub name: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
}
pub const COLLECTION_INFO: Item<CollectionInfo> = Item::new("collection_info");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
}
pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionConfig {
    pub per_address_limit: Option<u32>,
    pub start_time: Option<Timestamp>,
    pub max_token_limit: Option<u32>,
}
pub const COLLECTION_CONFIG: Item<CollectionConfig> = Item::new("collection_config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Contracts {
    pub metadata: Option<Addr>,
    pub whitelist: Option<Addr>,
    pub royalty: Option<Addr>,
}
pub const CONTRACTS: Item<Contracts> = Item::new("contracts");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Locks {
    pub burn_lock: bool,
    pub mint_lock: bool,
    pub transfer_lock: bool,
    pub send_lock: bool,
}
pub const LOCKS: Item<Locks> = Item::new("locks");

pub const TOKEN_LOCKS: Map<&str, Locks> = Map::new("token_locks");

pub const OPERATION_LOCK: Item<bool> = Item::new("operation_lock");

pub const TOKEN_IDS: Item<u32> = Item::new("token_ids");

pub const MINTED_TOKEN_AMOUNTS: Map<&str, u32> = Map::new("minted_token_amounts");

pub const MINT_MODULE_ADDR: Item<Addr> = Item::new("mint_module_addr");
