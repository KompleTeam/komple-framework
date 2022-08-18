use komple_types::shared::{CONFIG_NAMESPACE, COLLECTION_ADDR_NAMESPACE, OPERATORS_NAMESPACE};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub merge_lock: bool,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

pub const COLLECTION_ADDR: Item<Addr> = Item::new(COLLECTION_ADDR_NAMESPACE);

pub const OPERATORS: Item<Vec<Addr>> = Item::new(OPERATORS_NAMESPACE);
