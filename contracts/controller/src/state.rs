use rift_types::module::MODULE_ADDRS_NAMESPACE;
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
pub const CONTROLLER_INFO: Item<ControllerInfo> = Item::new("controller_info");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
}
pub const CONFIG: Item<Config> = Item::new("config");

pub const MODULE_ADDR: Map<&str, Addr> = Map::new(MODULE_ADDRS_NAMESPACE);
