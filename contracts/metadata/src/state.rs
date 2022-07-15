use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub admin: Addr,
    pub update_lock: bool,
}
pub const CONFIG: Item<Config> = Item::new("config");

pub const METADATA_LOCK: Map<&str, bool> = Map::new("metadata_lock");

pub const COLLECTION_ADDR: Item<Addr> = Item::new("collection_addr");

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Trait {
    // pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Metadata {
    pub image: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
}
pub const METADATA: Map<&str, Metadata> = Map::new("metadata");

// token_id - trait_type -> value
pub const ATTRIBUTES: Map<(&str, &str), String> = Map::new("attributes");
