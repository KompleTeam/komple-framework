use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};
use komple_types::{
    shared::CONFIG_NAMESPACE,
    whitelist::{WHITELIST_CONFIG_NAMESPACE, WHITELIST_NAMESPACE},
};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

#[cw_serde]
pub struct WhitelistConfig {
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub unit_price: Uint128,
    pub per_address_limit: u8,
    pub member_limit: u16,
    pub member_num: u16,
}
pub const WHITELIST_CONFIG: Item<WhitelistConfig> = Item::new(WHITELIST_CONFIG_NAMESPACE);

pub const WHITELIST: Map<Addr, bool> = Map::new(WHITELIST_NAMESPACE);
