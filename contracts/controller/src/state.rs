use komple_types::{
    controller::{CONTROLLER_INFO_NAMESPACE, WEBSITE_CONFIG_NAMESPACE}, module::MODULE_ADDRS_NAMESPACE, shared::CONFIG_NAMESPACE,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ControllerInfo {
    pub name: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
}
pub const CONTROLLER_INFO: Item<ControllerInfo> = Item::new(CONTROLLER_INFO_NAMESPACE);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

pub const MODULE_ADDR: Map<&str, Addr> = Map::new(MODULE_ADDRS_NAMESPACE);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WebsiteConfig {
    pub background_color: Option<String>,
    pub background_image: Option<String>,
    pub banner_image: Option<String>,
};
pub const WEBSITE_CONFIG: Item<WebsiteConfig> = Item::new(WEBSITE_CONFIG_NAMESPACE);