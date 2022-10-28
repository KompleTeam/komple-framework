use cosmwasm_schema::cw_serde;
use komple_types::{
    marketplace::FIXED_LISTING_NAMESPACE,
    shared::{CONFIG_NAMESPACE, HUB_ADDR_NAMESPACE, OPERATORS_NAMESPACE},
};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub native_denom: String,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

/// Hub module address.
pub const HUB_ADDR: Item<Addr> = Item::new(HUB_ADDR_NAMESPACE);

#[cw_serde]
pub struct FixedListing {
    pub collection_id: u32,
    pub token_id: u32,
    pub price: Uint128,
    pub owner: Addr,
}
/// Storage map for the fixed listings.
/// 
/// Collection id and token id are used as the key.
/// `FixedListing` is the value.
pub const FIXED_LISTING: Map<(u32, u32), FixedListing> = Map::new(FIXED_LISTING_NAMESPACE);

/// Operators of this contract.
pub const OPERATORS: Item<Vec<Addr>> = Item::new(OPERATORS_NAMESPACE);
