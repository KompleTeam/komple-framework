use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub main_collection: Option<u32>,
    pub controller_addr: Addr,
}
pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PasscardInfo {
    pub name: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
    pub total_num: Option<u16>,
}
pub const PASSCARD_INFO: Item<PasscardInfo> = Item::new("passcard_info");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Passcard {
    pub id: u16,
    pub price: Uint128,
    pub on_sale: bool,
    pub owner: Addr,
}
pub const PASSCARDS: Item<Passcard> = Item::new("passcards");

pub const MINTABLE_PASSCARDS: Map<(u32, u16), bool> = Map::new("available_passcards");

pub const PASSCARD_IDS: Map<u32, u16> = Map::new("passcard_ids");
