use cosmwasm_std::Decimal;
use komple_types::royalty::Royalty;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
    pub share: Decimal,
    pub royalty_type: Royalty,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateOwnerRoyaltyAddress {
        address: String,
    },
    UpdateTokenRoyaltyAddress {
        collection_id: u32,
        token_id: u32,
        address: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Royalty {
        owner: String,
        collection_id: u32,
        token_id: u32,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RoyaltyResponse {
    pub share: Decimal,
    pub address: String,
}
