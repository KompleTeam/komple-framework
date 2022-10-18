use cosmwasm_schema::cw_serde;
use komple_types::{
    fee::{FixedPayment, PercentagePayment, FIXED_FEES_NAMESPACE, PERCENTAGE_FEES_NAMESPACE},
    shared::{CONFIG_NAMESPACE, HUB_ADDR_NAMESPACE},
};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

// This is used for percentage fees meaning we have decimals
// (module_name, fee_name) -> decimal
// (hub_module, creation_fee) -> 0.1 (%10)
pub const PERCENTAGE_FEES: Map<(&str, &str), PercentagePayment> =
    Map::new(PERCENTAGE_FEES_NAMESPACE);

// This is used for fixed fees meaning constant values
// (module_name, fee_name) -> constant value
// (hub_module, creation_fee) -> 1_000_000 utoken
pub const FIXED_FEES: Map<(&str, &str), FixedPayment> = Map::new(FIXED_FEES_NAMESPACE);

pub const HUB_ADDR: Item<Addr> = Item::new(HUB_ADDR_NAMESPACE);
