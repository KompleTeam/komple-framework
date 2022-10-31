use cosmwasm_schema::cw_serde;
use komple_types::shared::{
    CONFIG_NAMESPACE, EXECUTE_LOCK_NAMESPACE, HUB_ADDR_NAMESPACE, OPERATORS_NAMESPACE,
};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub merge_lock: bool,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

/// Hub module address.
pub const HUB_ADDR: Item<Addr> = Item::new(HUB_ADDR_NAMESPACE);

/// Operators of this contract.
pub const OPERATORS: Item<Vec<Addr>> = Item::new(OPERATORS_NAMESPACE);

/// Lock for the execute entry point.
pub const EXECUTE_LOCK: Item<bool> = Item::new(EXECUTE_LOCK_NAMESPACE);
