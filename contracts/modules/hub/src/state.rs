use cosmwasm_schema::cw_serde;
use komple_types::{
    hub::{
        HUB_INFO_NAMESPACE, MARBU_FEE_MODULE_NAMESPACE, MODULE_ID_NAMESPACE,
        MODULE_TO_REGISTER_NAMESPACE, WEBSITE_CONFIG_NAMESPACE,
    },
    module::MODULE_ADDRS_NAMESPACE,
    shared::{CONFIG_NAMESPACE, OPERATORS_NAMESPACE},
};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

// These info also used for website profile
#[cw_serde]
pub struct HubInfo {
    pub name: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
}
pub const HUB_INFO: Item<HubInfo> = Item::new(HUB_INFO_NAMESPACE);

#[cw_serde]
pub struct Config {
    pub admin: Addr,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

pub const MODULE_ADDRS: Map<&str, Addr> = Map::new(MODULE_ADDRS_NAMESPACE);

// Contains the configuration for website profile
#[cw_serde]
pub struct WebsiteConfig {
    pub background_color: Option<String>,
    pub background_image: Option<String>,
    pub banner_image: Option<String>,
}
pub const WEBSITE_CONFIG: Item<WebsiteConfig> = Item::new(WEBSITE_CONFIG_NAMESPACE);

pub const OPERATORS: Item<Vec<Addr>> = Item::new(OPERATORS_NAMESPACE);

pub const MODULE_ID: Item<u64> = Item::new(MODULE_ID_NAMESPACE);

pub const MODULE_TO_REGISTER: Item<String> = Item::new(MODULE_TO_REGISTER_NAMESPACE);

pub const MARBU_FEE_MODULE: Item<Addr> = Item::new(MARBU_FEE_MODULE_NAMESPACE);
