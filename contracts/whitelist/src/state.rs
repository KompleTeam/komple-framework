use komple_types::{
    shared::CONFIG_NAMESPACE,
    whitelist::{WHITELIST_CONFIG_NAMESPACE, WHITELIST_NAMESPACE},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Timestamp};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistConfig {
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub unit_price: Coin,
    pub per_address_limit: u8,
    pub member_limit: u16,
    pub member_num: u16,
}
pub const WHITELIST_CONFIG: Item<WhitelistConfig> = Item::new(WHITELIST_CONFIG_NAMESPACE);

pub const WHITELIST: Map<Addr, bool> = Map::new(WHITELIST_NAMESPACE);
