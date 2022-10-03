use crate::state::{Config, FixedListing};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use komple_types::{marketplace::Listing, query::ResponseWrapper};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub native_denom: String,
}

#[cw_serde]
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

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ResponseWrapper<Config>)]
    Config {},
    #[returns(ResponseWrapper<FixedListing>)]
    FixedListing { collection_id: u32, token_id: u32 },
    #[returns(ResponseWrapper<Vec<FixedListing>>)]
    FixedListings {
        collection_id: u32,
        start_after: Option<u32>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
