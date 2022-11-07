use cosmwasm_schema::cw_serde;
use komple_types::{
    mint::{
        Collections, BLACKLIST_COLLECTION_ADDRS_NAMESPACE, COLLECTION_ADDRS_NAMESPACE,
        COLLECTION_ID_NAMESPACE, COLLECTION_INFO_NAMESPACE, CREATORS_NAMESPACE,
        LINKED_COLLECTIONS_NAMESPACE,
    },
    shared::{
        CONFIG_NAMESPACE, EXECUTE_LOCK_NAMESPACE, OPERATORS_NAMESPACE, PARENT_ADDR_NAMESPACE,
    },
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

/// General information about the collection.
#[cw_serde]
pub struct CollectionInfo {
    pub collection_type: Collections,
    pub name: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
    pub native_denom: String,
}
/// Map of collection ids to collection infos.
pub const COLLECTION_INFO: Map<u32, CollectionInfo> = Map::new(COLLECTION_INFO_NAMESPACE);

/// Map of collection ids to collection addresses.
pub const COLLECTION_ADDRS: Map<u32, Addr> = Map::new(COLLECTION_ADDRS_NAMESPACE);

/// ID used for the collection numbers.
pub const COLLECTION_ID: Item<u32> = Item::new(COLLECTION_ID_NAMESPACE);

/// Hub module address.
pub const HUB_ADDR: Item<Addr> = Item::new(PARENT_ADDR_NAMESPACE);

/// Operators of this contract.
pub const OPERATORS: Item<Vec<Addr>> = Item::new(OPERATORS_NAMESPACE);

/// Map of collection ids to a list of collection ids.
///
/// This is used to link collections to other
/// collections for custom usage in operations.
pub const LINKED_COLLECTIONS: Map<u32, Vec<u32>> = Map::new(LINKED_COLLECTIONS_NAMESPACE);

/// Map of collection ids to a list of collection ids.
///
/// This is used to blacklist collections and
/// prevent them from being used in operations.
pub const BLACKLIST_COLLECTION_ADDRS: Map<u32, Addr> =
    Map::new(BLACKLIST_COLLECTION_ADDRS_NAMESPACE);

/// Lock for the execute entry point.
pub const EXECUTE_LOCK: Item<bool> = Item::new(EXECUTE_LOCK_NAMESPACE);

/// List of creators that can create collections.
pub const CREATORS: Item<Vec<Addr>> = Item::new(CREATORS_NAMESPACE);
