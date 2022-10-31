use cosmwasm_schema::cw_serde;

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use komple_types::{
    permission::{
        MODULE_PERMISSIONS_NAMESPACE, PERMISSIONS_NAMESPACE, PERMISSION_ID_NAMESPACE,
        PERMISSION_TO_REGISTER_NAMESPACE,
    },
    shared::{CONFIG_NAMESPACE, EXECUTE_LOCK_NAMESPACE, HUB_ADDR_NAMESPACE, OPERATORS_NAMESPACE},
};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

pub const HUB_ADDR: Item<Addr> = Item::new(HUB_ADDR_NAMESPACE);

pub const MODULE_PERMISSIONS: Map<&str, Vec<String>> = Map::new(MODULE_PERMISSIONS_NAMESPACE);

pub const OPERATORS: Item<Vec<Addr>> = Item::new(OPERATORS_NAMESPACE);

// Used as a temporary variable for saving permission to be registered
pub const PERMISSION_TO_REGISTER: Item<String> = Item::new(PERMISSION_TO_REGISTER_NAMESPACE);

// Used for registering permissions
pub const PERMISSION_ID: Item<u64> = Item::new(PERMISSION_ID_NAMESPACE);

// Used for matching permission names to addresses
pub const PERMISSIONS: Map<&str, Addr> = Map::new(PERMISSIONS_NAMESPACE);

/// Lock for the execute entry point.
pub const EXECUTE_LOCK: Item<bool> = Item::new(EXECUTE_LOCK_NAMESPACE);
