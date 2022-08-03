use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use komple_types::permission::Permissions;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
}
pub const CONFIG: Item<Config> = Item::new("config");

pub const CONTROLLER_ADDR: Item<Addr> = Item::new("controller_addr");

pub const MODULE_PERMISSIONS: Map<&str, Vec<Permissions>> = Map::new("module_permissions");

pub const OPERATORS: Item<Vec<Addr>> = Item::new("operators");
