use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub main_collection: Option<u32>,
    pub controller_addr: Addr,
}
pub const CONFIG: Item<Config> = Item::new("config");
