use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{Metadata, Trait};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddMetadata {
        token_id: String,
        metadata: Metadata,
        attributes: Vec<Trait>,
    },
    UpdateMetadata {
        token_id: String,
        metadata: Metadata,
    },
    AddAttribute {
        token_id: String,
        attribute: Trait,
    },
    UpdateAttribute {
        token_id: String,
        attribute: Trait,
    },
    RemoveAttribute {
        token_id: String,
        trait_type: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Metadata { token_id: String },
    MetadataLock { token_id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MetadataResponse {
    pub metadata: Metadata,
    pub attributes: Vec<Trait>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LockResponse {
    pub locked: bool,
}
