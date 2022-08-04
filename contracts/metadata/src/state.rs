use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::{Item, Map};

use rift_types::metadata::Metadata as MetadataType;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub admin: Addr,
    pub update_lock: bool,
    pub metadata_type: MetadataType,
}
pub const CONFIG: Item<Config> = Item::new("config");

// pub const METADATA_LOCK: Map<&str, bool> = Map::new("metadata_lock");

pub const COLLECTION_ADDR: Item<Addr> = Item::new("collection_addr");

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Trait {
    // pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct MetaInfo {
    pub image: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Metadata {
    pub meta_info: MetaInfo,
    pub attributes: Vec<Trait>,
}
pub const METADATA: Map<u32, Metadata> = Map::new("metadata");

pub const METADATA_ID: Item<u32> = Item::new("metadata_id");

pub const STATIC_METADATA: Map<u32, u32> = Map::new("static_metadata");

pub const DYNAMIC_METADATA: Map<u32, Metadata> = Map::new("dynamic_metadata");
