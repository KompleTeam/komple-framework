use cosmwasm_std::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use token::state::Locks;

use crate::state::CollectionInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub symbol: String,
    pub minter: String,
    pub token_code_id: u64,
    pub collection_info: CollectionInfo,
    pub per_address_limit: Option<u32>,
    pub whitelist: Option<String>,
    pub start_time: Option<Timestamp>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateLocks { locks: Locks },
    Mint { recipient: String, token_id: u32 },
    SetWhitelist { whitelist: Option<String> },
    UpdateStartTime(Option<Timestamp>),
    UpdatePerAddressLimit { per_address_limit: Option<u32> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {}
