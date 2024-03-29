use cosmwasm_schema::cw_serde;

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use komple_framework_types::modules::permission::{
    MODULE_PERMISSIONS_NAMESPACE, PERMISSIONS_NAMESPACE, PERMISSION_ID_NAMESPACE,
    PERMISSION_TO_REGISTER_NAMESPACE,
};
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

/// List of permissions that are set to a module.
pub const MODULE_PERMISSIONS: Map<&str, Vec<String>> = Map::new(MODULE_PERMISSIONS_NAMESPACE);

/// Operators of this contract.
pub const OPERATORS: Item<Vec<Addr>> = Item::new(OPERATORS_NAMESPACE);

/// Permission name to register. This is utilized in the reply handler of this contract.
pub const PERMISSION_TO_REGISTER: Item<String> = Item::new(PERMISSION_TO_REGISTER_NAMESPACE);

/// ID used for the permission registration purposes.
pub const PERMISSION_ID: Item<u64> = Item::new(PERMISSION_ID_NAMESPACE);

/// Addresses of the registered permissions.
pub const PERMISSIONS: Map<&str, Addr> = Map::new(PERMISSIONS_NAMESPACE);

/// Lock for the execute entry point.
pub const EXECUTE_LOCK: Item<bool> = Item::new(EXECUTE_LOCK_NAMESPACE);
