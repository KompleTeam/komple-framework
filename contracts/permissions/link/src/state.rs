use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use komple_types::{permission::PERMISSION_MODULE_ADDR_NAMESPACE, shared::CONFIG_NAMESPACE};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
}

pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

pub const PERMISSION_MODULE_ADDR: Item<Addr> = Item::new(PERMISSION_MODULE_ADDR_NAMESPACE);
