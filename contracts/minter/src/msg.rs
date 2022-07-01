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
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateLocks { locks: Locks },
    Mint {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {}
