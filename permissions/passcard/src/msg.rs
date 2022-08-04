use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{Passcard, PasscardInfo};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub controller_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddPasscard {
        collection_id: u32,
        base_price: Uint128,
        passcard_info: PasscardInfo,
    },
}

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
    pub total_num: u16,
    pub passcards: Vec<Passcard>,
}
