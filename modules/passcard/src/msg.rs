use cosmwasm_std::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use token_contract::{msg::TokenInfo, state::CollectionInfo};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreatePasscard {
        code_id: u64,
        collection_info: CollectionInfo,
        token_info: TokenInfo,
        per_address_limit: Option<u32>,
        start_time: Option<Timestamp>,
        whitelist: Option<String>,
        royalty: Option<String>,
        main_collections: Vec<u32>,
        max_token_limit: Option<u32>,
    },
    Mint {
        passcard_id: u32,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    PasscardAddress { passcard_id: u32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AddressResponse {
    pub address: String,
}
