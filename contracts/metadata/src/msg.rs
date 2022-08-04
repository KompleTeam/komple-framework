use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{MetaInfo, Trait};

use rift_types::metadata::Metadata as MetadataType;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
    pub metadata_type: MetadataType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddMetadata {
        meta_info: MetaInfo,
        attributes: Vec<Trait>,
    },
    LinkMetadata {
        token_id: u32,
        metadata_id: Option<u32>,
    },
    UpdateMetaInfo {
        token_id: u32,
        meta_info: MetaInfo,
    },
    AddAttribute {
        token_id: u32,
        attribute: Trait,
    },
    UpdateAttribute {
        token_id: u32,
        attribute: Trait,
    },
    RemoveAttribute {
        token_id: u32,
        trait_type: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    RawMetadata { metadata_id: u32 },
    Metadata { token_id: u32 },
    // MetadataLock { token_id: u32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LockResponse {
    pub locked: bool,
}
