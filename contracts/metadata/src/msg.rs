use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{MetaInfo, Metadata, Trait};

use komple_types::metadata::Metadata as MetadataType;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
    pub metadata_type: MetadataType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // Add metadata to the contract for linking it to a token
    // based on metadata type
    AddMetadata {
        meta_info: MetaInfo,
        attributes: Vec<Trait>,
    },
    // Link metadata to a token on minting
    LinkMetadata {
        token_id: u32,
        metadata_id: Option<u32>,
    },
    // Unlink metadata from a token
    UnlinkMetadata {
        token_id: u32,
    },
    // Update the meta info for a metadata
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
    RawMetadata {
        metadata_id: u32,
    },
    Metadata {
        token_id: u32,
    },
    RawMetadatas {
        start_after: Option<u32>,
        limit: Option<u8>,
    },
    Metadatas {
        metadata_type: MetadataType,
        start_after: Option<u32>,
        limit: Option<u8>,
    },
    // MetadataLock { token_id: u32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MetadataResponse {
    pub metadata: Metadata,
    pub metadata_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
