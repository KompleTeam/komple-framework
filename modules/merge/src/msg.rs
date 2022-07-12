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
pub enum MergeAction {
    Mint,
    Burn,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MergeMsg {
    pub collection_type: Collections,
    pub collection_id: u32,
    pub action: MergeAction,
    pub owner: Option<String>,
    pub token_id: Option<u32>,
}
