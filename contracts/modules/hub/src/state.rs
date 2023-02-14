use cosmwasm_schema::cw_serde;
use komple_framework_types::shared::{CONFIG_NAMESPACE, OPERATORS_NAMESPACE};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use komple_framework_types::modules::hub::{
    HUB_INFO_NAMESPACE, MARBU_FEE_MODULE_NAMESPACE, MODULE_ID_NAMESPACE,
    MODULE_TO_REGISTER_NAMESPACE,
};
use komple_framework_types::modules::MODULES_NAMESPACE;

/// General information about the hub module.
/// This information is equal to project information.
#[cw_serde]
pub struct HubInfo {
    pub name: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
}
pub const HUB_INFO: Item<HubInfo> = Item::new(HUB_INFO_NAMESPACE);

#[cw_serde]
pub struct Config {
    pub admin: Addr,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

/// Addresses of the registered modules.
pub const MODULES: Map<String, Addr> = Map::new(MODULES_NAMESPACE);

/// Operators of this contract.
pub const OPERATORS: Item<Vec<Addr>> = Item::new(OPERATORS_NAMESPACE);

/// ID used for the module registration purposes.
pub const MODULE_ID: Item<u64> = Item::new(MODULE_ID_NAMESPACE);

/// Module name to register. This is utilized in the reply handler of this contract.
pub const MODULE_TO_REGISTER: Item<String> = Item::new(MODULE_TO_REGISTER_NAMESPACE);

/// Fee module address if hub is created through Marbu.
pub const MARBU_FEE_MODULE: Item<Addr> = Item::new(MARBU_FEE_MODULE_NAMESPACE);
