use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub merge_lock: bool,
}
pub const CONFIG: Item<Config> = Item::new("config");

pub const CONTROLLER_ADDR: Item<Addr> = Item::new("controller_addr");

pub const OPERATORS: Item<Vec<Addr>> = Item::new("operators");
