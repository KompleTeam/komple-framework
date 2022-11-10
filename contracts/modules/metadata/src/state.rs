use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use komple_types::{
    shared::{CONFIG_NAMESPACE, OPERATORS_NAMESPACE, PARENT_ADDR_NAMESPACE},
};
use komple_types::modules::metadata::{
    DYNAMIC_LINKED_METADATA_NAMESPACE, LINKED_METADATA_NAMESPACE, Metadata as MetadataType,
    METADATA_ID_NAMESPACE, METADATA_NAMESPACE,
};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub metadata_type: MetadataType,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

// pub const METADATA_LOCK: Map<&str, bool> = Map::new("metadata_lock");

/// Address of the token contract.
pub const COLLECTION_ADDR: Item<Addr> = Item::new(PARENT_ADDR_NAMESPACE);

#[cw_serde]
pub struct Trait {
    // pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}
#[cw_serde]
pub struct MetaInfo {
    pub image: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
}
#[cw_serde]
pub struct Metadata {
    pub meta_info: MetaInfo,
    pub attributes: Vec<Trait>,
}
/// Raw metadata values that is saved to storage.
/// `AddMetadata` message can be used to add items to this map.
pub const METADATA: Map<u32, Metadata> = Map::new(METADATA_NAMESPACE);

/// ID used to identify a single raw metadata.
pub const METADATA_ID: Item<u32> = Item::new(METADATA_ID_NAMESPACE);

/// Linked metadata values that is saved to storage.
/// `LinkMetadata` message can be used to add items to this map.
///
/// Raw metadata ids are mapped to token ids.
/// Only works for `Standard` and `Shared` metadata types.
pub const LINKED_METADATA: Map<u32, u32> = Map::new(LINKED_METADATA_NAMESPACE);

/// Linked metadata values that is saved to storage.
/// `LinkMetadata` message can be used to add items to this map.
///
/// Whole metadata objects are mapped to token ids.
/// Only works for `Dynamic` metadata type.
pub const DYNAMIC_LINKED_METADATA: Map<u32, Metadata> = Map::new(DYNAMIC_LINKED_METADATA_NAMESPACE);

/// Operators of this contract.
pub const OPERATORS: Item<Vec<Addr>> = Item::new(OPERATORS_NAMESPACE);
