use crate::state::Config;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;
use komple_types::query::ResponseWrapper;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    AddShare {
        name: String,
        address: Option<String>,
        percentage: Decimal,
    },
    UpdateShare {
        name: String,
        address: Option<String>,
        percentage: Decimal,
    },
    RemoveShare {
        name: String,
    },
    Distribute {
        custom_addresses: Option<Vec<CustomAddress>>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ResponseWrapper<Config>)]
    Config {},
    #[returns(ResponseWrapper<ShareResponse>)]
    Share { name: String },
    #[returns(ResponseWrapper<Vec<ShareResponse>>)]
    Shares {},
    #[returns(ResponseWrapper<Decimal>)]
    TotalFee {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ShareResponse {
    pub name: String,
    pub payment_address: Option<String>,
    pub fee_percentage: Decimal,
}

#[cw_serde]
pub struct CustomAddress {
    pub name: String,
    pub payment_address: String,
}
