use cosmwasm_schema::cw_serde;
use komple_framework_types::shared::{
    CONFIG_NAMESPACE, EXECUTE_LOCK_NAMESPACE, OPERATORS_NAMESPACE, PARENT_ADDR_NAMESPACE,
};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use komple_framework_types::modules::fee::{FundInfo, FUND_INFO_NAMESPACE};
use komple_framework_types::modules::marketplace::FIXED_LISTING_NAMESPACE;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub buy_lock: bool,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

/// Hub module address.
pub const HUB_ADDR: Item<Addr> = Item::new(PARENT_ADDR_NAMESPACE);

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

/// Lock for the execute entry point.
pub const EXECUTE_LOCK: Item<bool> = Item::new(EXECUTE_LOCK_NAMESPACE);

/// Fund info for the marketplace.
///
/// This is used to lock the marketplace with a specific fund info.
pub const FUND_INFO: Item<FundInfo> = Item::new(FUND_INFO_NAMESPACE);
