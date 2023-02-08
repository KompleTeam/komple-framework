use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

use komple_framework_types::shared::{
    CONFIG_NAMESPACE, EXECUTE_LOCK_NAMESPACE, OPERATORS_NAMESPACE, PARENT_ADDR_NAMESPACE,
};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

/// Hub module address.
pub const HUB_ADDR: Item<Addr> = Item::new(PARENT_ADDR_NAMESPACE);

/// Operators of this contract.
pub const OPERATORS: Item<Vec<Addr>> = Item::new(OPERATORS_NAMESPACE);

/// Lock for the execute entry point.
pub const EXECUTE_LOCK: Item<bool> = Item::new(EXECUTE_LOCK_NAMESPACE);
