use cosmwasm_std::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use token::{msg::TokenInfo, state::CollectionInfo};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateCollection {
        code_id: u64,
        collection_info: CollectionInfo,
        token_info: TokenInfo,
        per_address_limit: Option<u32>,
        start_time: Option<Timestamp>,
        whitelist: Option<String>,
    },
    UpdateMintLock {
        lock: bool,
    },
    Mint {
        collection_id: u32,
    },
    MintTo {
        collection_id: u32,
        recipient: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    CollectionAddress { collection_id: u32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokenAddressResponse {
    pub token_address: String,
}
