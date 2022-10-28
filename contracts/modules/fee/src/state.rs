use cosmwasm_schema::cw_serde;
use komple_types::{
    fee::{FixedPayment, PercentagePayment, FIXED_FEES_NAMESPACE, PERCENTAGE_FEES_NAMESPACE},
    shared::{CONFIG_NAMESPACE, HUB_ADDR_NAMESPACE},
};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

/// General config for the contract.
#[cw_serde]
pub struct Config {
    pub admin: Addr,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

/// The fees that are percentage.
///
/// Module name and fee name are used as the key.
/// ```PercentagePayment``` is the value.
pub const PERCENTAGE_FEES: Map<(&str, &str), PercentagePayment> =
    Map::new(PERCENTAGE_FEES_NAMESPACE);

/// The fees that are fixed.
///
/// Module name and fee name are used as the key.
/// ```FixedPayment``` is the value.
pub const FIXED_FEES: Map<(&str, &str), FixedPayment> = Map::new(FIXED_FEES_NAMESPACE);

/// Hub module address.
pub const HUB_ADDR: Item<Addr> = Item::new(HUB_ADDR_NAMESPACE);
