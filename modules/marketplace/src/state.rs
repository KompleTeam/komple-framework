use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub fee_percentage: Decimal,
    pub native_denom: String,
}
pub const CONFIG: Item<Config> = Item::new("config");

pub const CONTROLLER_ADDR: Item<Addr> = Item::new("controller_addr");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FixedListing {
    pub collection_id: u32,
    pub token_id: u32,
    pub price: Uint128,
    pub owner: Addr,
}
pub const FIXED_LISTING: Map<(u32, u32), FixedListing> = Map::new("fixed_listing");
