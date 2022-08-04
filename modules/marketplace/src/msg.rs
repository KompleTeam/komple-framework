use cosmwasm_std::Uint128;
use rift_types::marketplace::Listing;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
    pub native_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ListFixedToken {
        collection_id: u32,
        token_id: u32,
        price: Uint128,
    },
    DelistFixedToken {
        collection_id: u32,
        token_id: u32,
    },
    UpdatePrice {
        listing_type: Listing,
        collection_id: u32,
        token_id: u32,
        price: Uint128,
    },
    Buy {
        listing_type: Listing,
        collection_id: u32,
        token_id: u32,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    FixedListing { collection_id: u32, token_id: u32 },
}
