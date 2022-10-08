use komple_types::{
    collection::Collections,
    shared::{CONFIG_NAMESPACE, OPERATORS_NAMESPACE},
    tokens::{
        Locks, COLLECTION_CONFIG_NAMESPACE, COLLECTION_INFO_NAMESPACE, CONTRACTS_NAMESPACE,
        LOCKS_NAMESPACE, MINTED_TOKENS_PER_ADDR_NAMESPACE, MINT_MODULE_ADDR_NAMESPACE,
        TOKEN_IDS_NAMESPACE, TOKEN_LOCKS_NAMESPACE,
    },
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionInfo {
    pub collection_type: Collections,
    pub name: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
}
pub const COLLECTION_INFO: Item<CollectionInfo> = Item::new(COLLECTION_INFO_NAMESPACE);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub creator: Addr,
    pub royalty_share: Option<Decimal>,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionConfig {
    pub native_denom: String,
    pub per_address_limit: Option<u32>,
    pub start_time: Option<Timestamp>,
    pub max_token_limit: Option<u32>,
    pub unit_price: Option<Uint128>,
    pub ipfs_link: Option<String>,
}
pub const COLLECTION_CONFIG: Item<CollectionConfig> = Item::new(COLLECTION_CONFIG_NAMESPACE);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Contracts {
    pub metadata: Option<Addr>,
    pub whitelist: Option<Addr>,
}
pub const CONTRACTS: Item<Contracts> = Item::new(CONTRACTS_NAMESPACE);

pub const LOCKS: Item<Locks> = Item::new(LOCKS_NAMESPACE);

pub const TOKEN_LOCKS: Map<&str, Locks> = Map::new(TOKEN_LOCKS_NAMESPACE);

pub const TOKEN_IDS: Item<u32> = Item::new(TOKEN_IDS_NAMESPACE);

pub const MINTED_TOKENS_PER_ADDR: Map<&str, u32> = Map::new(MINTED_TOKENS_PER_ADDR_NAMESPACE);

pub const MINT_MODULE_ADDR: Item<Addr> = Item::new(MINT_MODULE_ADDR_NAMESPACE);

pub const OPERATORS: Item<Vec<Addr>> = Item::new(OPERATORS_NAMESPACE);
