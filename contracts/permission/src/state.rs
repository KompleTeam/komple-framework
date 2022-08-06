use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use komple_types::{
    permission::{Permissions, MODULE_PERMISSIONS_NAMESPACE},
    shared::{CONFIG_NAMESPACE, CONTROLLER_ADDR_NAMESPACE, OPERATORS_NAMESPACE},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

pub const CONTROLLER_ADDR: Item<Addr> = Item::new(CONTROLLER_ADDR_NAMESPACE);

pub const MODULE_PERMISSIONS: Map<&str, Vec<Permissions>> = Map::new(MODULE_PERMISSIONS_NAMESPACE);

pub const OPERATORS: Item<Vec<Addr>> = Item::new(OPERATORS_NAMESPACE);
