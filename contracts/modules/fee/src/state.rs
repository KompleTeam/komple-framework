use cosmwasm_schema::cw_serde;
use komple_framework_types::shared::{
    CONFIG_NAMESPACE, EXECUTE_LOCK_NAMESPACE, OPERATORS_NAMESPACE, PARENT_ADDR_NAMESPACE,
};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use komple_framework_types::modules::fee::{
    FixedPayment, PercentagePayment, FIXED_FEES_NAMESPACE, PERCENTAGE_FEES_NAMESPACE,
};

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
pub const HUB_ADDR: Item<Addr> = Item::new(PARENT_ADDR_NAMESPACE);

/// Lock for the execute entry point.
pub const EXECUTE_LOCK: Item<bool> = Item::new(EXECUTE_LOCK_NAMESPACE);

/// Operators of this contract.
pub const OPERATORS: Item<Vec<Addr>> = Item::new(OPERATORS_NAMESPACE);
