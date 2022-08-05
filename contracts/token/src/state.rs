use komple_types::{
    collection::Collections,
    tokens::{Locks, LOCKS_NAMESPACE, TOKEN_LOCKS_NAMESPACE},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Decimal, Timestamp};
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
    pub native_denom: String,
    pub royalty_share: Option<Decimal>,
}
pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionConfig {
    pub per_address_limit: Option<u32>,
    pub start_time: Option<Timestamp>,
    pub max_token_limit: Option<u32>,
    pub unit_price: Option<Coin>,
}
pub const COLLECTION_CONFIG: Item<CollectionConfig> = Item::new("collection_config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Contracts {
    pub metadata: Option<Addr>,
    pub whitelist: Option<Addr>,
}
pub const CONTRACTS: Item<Contracts> = Item::new("contracts");

pub const LOCKS: Item<Locks> = Item::new(LOCKS_NAMESPACE);

pub const TOKEN_LOCKS: Map<&str, Locks> = Map::new(TOKEN_LOCKS_NAMESPACE);

pub const TOKEN_IDS: Item<u32> = Item::new("token_ids");

pub const MINTED_TOKENS_PER_ADDR: Map<&str, u32> = Map::new("minted_tokens_per_addr");

pub const MINT_MODULE_ADDR: Item<Addr> = Item::new("mint_module_addr");

pub const OPERATORS: Item<Vec<Addr>> = Item::new("operators");
