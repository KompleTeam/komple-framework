use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::Passcard;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub controller_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    ListAvailablePasscards {
        collection_id: u32,
    },
    GetPasscard {
        collection_id: u32,
        passcard_id: u16,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ListPasscardsResponse {
    pub total_num: Option<u16>,
    pub passcards: Vec<Passcard>,
}
