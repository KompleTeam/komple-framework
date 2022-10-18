use cosmwasm_schema::cw_serde;
use komple_types::{
    collection::Collections,
    shared::{CONFIG_NAMESPACE, OPERATORS_NAMESPACE},
    token::{
        Locks, SubModules, COLLECTION_TYPE_NAMESPACE, LOCKS_NAMESPACE,
        MINTED_TOKENS_PER_ADDR_NAMESPACE, MINT_MODULE_ADDR_NAMESPACE, SUB_MODULES_NAMESPACE,
        TOKEN_IDS_NAMESPACE, TOKEN_LOCKS_NAMESPACE,
    },
};

use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub creator: Addr,
    pub per_address_limit: Option<u32>,
    pub start_time: Option<Timestamp>,
    pub max_token_limit: Option<u32>,
    pub ipfs_link: Option<String>,
}
pub const CONFIG: Item<Config> = Item::new(CONFIG_NAMESPACE);

#[cw_serde]
pub struct CollectionConfig {
    pub per_address_limit: Option<u32>,
    pub start_time: Option<Timestamp>,
    pub max_token_limit: Option<u32>,
    pub ipfs_link: Option<String>,
}

pub const SUB_MODULES: Item<SubModules> = Item::new(SUB_MODULES_NAMESPACE);

pub const LOCKS: Item<Locks> = Item::new(LOCKS_NAMESPACE);

pub const TOKEN_LOCKS: Map<&str, Locks> = Map::new(TOKEN_LOCKS_NAMESPACE);

pub const TOKEN_IDS: Item<u32> = Item::new(TOKEN_IDS_NAMESPACE);

pub const MINTED_TOKENS_PER_ADDR: Map<&str, u32> = Map::new(MINTED_TOKENS_PER_ADDR_NAMESPACE);

pub const MINT_MODULE_ADDR: Item<Addr> = Item::new(MINT_MODULE_ADDR_NAMESPACE);

pub const OPERATORS: Item<Vec<Addr>> = Item::new(OPERATORS_NAMESPACE);

pub const COLLECTION_TYPE: Item<Collections> = Item::new(COLLECTION_TYPE_NAMESPACE);