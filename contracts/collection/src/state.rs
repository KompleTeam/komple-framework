use komple_types::{
    collection::{COLLECTION_INFO_NAMESPACE, WEBSITE_CONFIG_NAMESPACE},
    module::MODULE_ADDRS_NAMESPACE,
    shared::CONFIG_NAMESPACE,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

// These info also used for website profile
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionInfo {
    pub name: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
}
pub const COLLECTION_INFO: Item<CollectionInfo> = Item::new(COLLECTION_INFO_NAMESPACE);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

pub const MODULE_ADDR: Map<&str, Addr> = Map::new(MODULE_ADDRS_NAMESPACE);

// Contains the configuration for website profile
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WebsiteConfig {
    pub background_color: Option<String>,
    pub background_image: Option<String>,
    pub banner_image: Option<String>,
}
pub const WEBSITE_CONFIG: Item<WebsiteConfig> = Item::new(WEBSITE_CONFIG_NAMESPACE);
