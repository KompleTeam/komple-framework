use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use komple_types::{
    metadata::{
        Metadata as MetadataType, COLLECTION_ADDR_NAMESPACE, DYNAMIC_LINKED_METADATA_NAMESPACE,
        LINKED_METADATA_NAMESPACE, METADATA_ID_NAMESPACE, METADATA_NAMESPACE,
    },
    shared::{CONFIG_NAMESPACE, OPERATORS_NAMESPACE},
};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub update_lock: bool,
    pub metadata_type: MetadataType,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

// pub const METADATA_LOCK: Map<&str, bool> = Map::new("metadata_lock");

pub const COLLECTION_ADDR: Item<Addr> = Item::new(COLLECTION_ADDR_NAMESPACE);

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
pub const METADATA: Map<u32, Metadata> = Map::new(METADATA_NAMESPACE);

pub const METADATA_ID: Item<u32> = Item::new(METADATA_ID_NAMESPACE);

pub const LINKED_METADATA: Map<u32, u32> = Map::new(LINKED_METADATA_NAMESPACE);

pub const DYNAMIC_LINKED_METADATA: Map<u32, Metadata> = Map::new(DYNAMIC_LINKED_METADATA_NAMESPACE);

pub const OPERATORS: Item<Vec<Addr>> = Item::new(OPERATORS_NAMESPACE);
