use cosmwasm_schema::cw_serde;
use komple_types::{
    fee::{FIXED_FEES_NAMESPACE, PERCENTAGE_FEES_NAMESPACE},
    shared::CONFIG_NAMESPACE,
};

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

#[cw_serde]
pub struct PercentagePayment {
    // Address is optional and can be overriden with a custom address on distribution
    pub address: Option<String>,
    pub value: Decimal,
}
// This is used for percentage fees meaning we have decimals
// (module_name, fee_name) -> decimal
// (hub_module, creation_fee) -> 0.1 (%10)
pub const PERCENTAGE_FEES: Map<(&str, &str), PercentagePayment> =
    Map::new(PERCENTAGE_FEES_NAMESPACE);

#[cw_serde]
pub struct FixedPayment {
    // Address is optional and can be overriden with a custom address on distribution
    pub address: Option<String>,
    pub value: Uint128,
}
// This is used for fixed fees meaning constant values
// (module_name, fee_name) -> constant value
// (hub_module, creation_fee) -> 1_000_000 utoken
pub const FIXED_FEES: Map<(&str, &str), FixedPayment> = Map::new(FIXED_FEES_NAMESPACE);
