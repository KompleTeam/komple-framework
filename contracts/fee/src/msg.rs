use cosmwasm_std::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Share { name: String },
    Shares {},
    TotalFee {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ShareResponse {
    pub name: String,
    pub payment_address: Option<String>,
    pub fee_percentage: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CustomAddress {
    pub name: String,
    pub payment_address: String,
}
