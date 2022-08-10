use komple_types::{
    collection::{
        COLLECTION_ADDRS_NAMESPACE, COLLECTION_ID_NAMESPACE, COLLECTION_TYPES_NAMESPACE,
        LINKED_COLLECTIONS_NAMESPACE,
    },
    shared::{CONFIG_NAMESPACE, CONTROLLER_ADDR_NAMESPACE, OPERATORS_NAMESPACE},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub public_collection_creation: bool,
    pub mint_lock: bool,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

pub const COLLECTION_ADDRS: Map<u32, Addr> = Map::new(COLLECTION_ADDRS_NAMESPACE);

pub const COLLECTION_ID: Item<u32> = Item::new(COLLECTION_ID_NAMESPACE);

pub const CONTROLLER_ADDR: Item<Addr> = Item::new(CONTROLLER_ADDR_NAMESPACE);

pub const OPERATORS: Item<Vec<Addr>> = Item::new(OPERATORS_NAMESPACE);

pub const COLLECTION_TYPES: Map<&str, Vec<u32>> = Map::new(COLLECTION_TYPES_NAMESPACE);

pub const LINKED_COLLECTIONS: Map<u32, Vec<u32>> = Map::new(LINKED_COLLECTIONS_NAMESPACE);
