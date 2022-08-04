use cosmwasm_std::Binary;
use rift_types::collection::Collections;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateMergeLock {
        lock: bool,
    },
    Merge {
        msg: Binary,
    },
    PermissionMerge {
        permission_msg: Binary,
        merge_msg: Binary,
    },
    UpdateWhitelistAddresses {
        addrs: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    WhitelistAddresses {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MergeBurnMsg {
    pub collection_id: u32,
    pub token_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MergeMsg {
    pub mint: Vec<u32>,
    pub burn: Vec<MergeBurnMsg>,
}
