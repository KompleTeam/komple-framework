use komple_types::{
    bundle::{
        BUNDLE_ADDRS_NAMESPACE, BUNDLE_ID_NAMESPACE, BUNDLE_TYPES_NAMESPACE,
        LINKED_BUNDLES_NAMESPACE,
    },
    shared::{CONFIG_NAMESPACE, HUB_ADDR_NAMESPACE, OPERATORS_NAMESPACE},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub public_bundle_creation: bool,
    pub mint_lock: bool,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

pub const BUNDLE_ADDRS: Map<u32, Addr> = Map::new(BUNDLE_ADDRS_NAMESPACE);

pub const BUNDLE_ID: Item<u32> = Item::new(BUNDLE_ID_NAMESPACE);

pub const HUB_ADDR: Item<Addr> = Item::new(HUB_ADDR_NAMESPACE);

pub const OPERATORS: Item<Vec<Addr>> = Item::new(OPERATORS_NAMESPACE);

pub const BUNDLE_TYPES: Map<&str, Vec<u32>> = Map::new(BUNDLE_TYPES_NAMESPACE);

pub const LINKED_BUNDLES: Map<u32, Vec<u32>> = Map::new(LINKED_BUNDLES_NAMESPACE);
