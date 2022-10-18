use cosmwasm_schema::cw_serde;
use komple_types::{
    collection::{
        Collections, BLACKLIST_COLLECTION_ADDRS_NAMESPACE, COLLECTION_ADDRS_NAMESPACE,
        COLLECTION_ID_NAMESPACE, COLLECTION_INFO_NAMESPACE, LINKED_COLLECTIONS_NAMESPACE,
    },
    shared::{CONFIG_NAMESPACE, HUB_ADDR_NAMESPACE, OPERATORS_NAMESPACE},
};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub public_collection_creation: bool,
    pub mint_lock: bool,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

#[cw_serde]
pub struct CollectionInfo {
    pub collection_type: Collections,
    pub name: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
    pub native_denom: String,
}
pub const COLLECTION_INFO: Map<u32, CollectionInfo> = Map::new(COLLECTION_INFO_NAMESPACE);

pub const COLLECTION_ADDRS: Map<u32, Addr> = Map::new(COLLECTION_ADDRS_NAMESPACE);

pub const COLLECTION_ID: Item<u32> = Item::new(COLLECTION_ID_NAMESPACE);

pub const HUB_ADDR: Item<Addr> = Item::new(HUB_ADDR_NAMESPACE);

pub const OPERATORS: Item<Vec<Addr>> = Item::new(OPERATORS_NAMESPACE);

pub const LINKED_COLLECTIONS: Map<u32, Vec<u32>> = Map::new(LINKED_COLLECTIONS_NAMESPACE);

pub const BLACKLIST_COLLECTION_ADDRS: Map<u32, Addr> =
    Map::new(BLACKLIST_COLLECTION_ADDRS_NAMESPACE);
