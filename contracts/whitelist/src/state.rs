use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Timestamp};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
}
pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistConfig {
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub unit_price: Coin,
    pub per_address_limit: u8,
    pub member_limit: u16,
    pub member_num: u16,
}
pub const WHITELIST_CONFIG: Item<WhitelistConfig> = Item::new("whitelist_config");

pub const WHITELIST: Map<Addr, bool> = Map::new("whitelist");
