use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};
use komple_types::modules::whitelist::WHITELIST_NAMESPACE;
use komple_types::shared::CONFIG_NAMESPACE;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub per_address_limit: u8,
    pub member_limit: u16,
    pub member_num: u16,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

#[cw_serde]
pub struct WhitelistConfig {
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub per_address_limit: u8,
    pub member_limit: u16,
}

pub const WHITELIST: Map<Addr, bool> = Map::new(WHITELIST_NAMESPACE);
