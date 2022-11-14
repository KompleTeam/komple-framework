use cosmwasm_schema::cw_serde;
use komple_framework_types::shared::{CONFIG_NAMESPACE, PARENT_ADDR_NAMESPACE};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

pub const PERMISSION_MODULE_ADDR: Item<Addr> = Item::new(PARENT_ADDR_NAMESPACE);
