use crate::state::{Config, MetaInfo, Metadata, Trait};
use cosmwasm_schema::{cw_serde, QueryResponses};
use komple_types::{metadata::Metadata as MetadataType, query::ResponseWrapper};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub metadata_type: MetadataType,
}

#[cw_serde]
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

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ResponseWrapper<Config>)]
    Config {},
    #[returns(ResponseWrapper<Metadata>)]
    RawMetadata { metadata_id: u32 },
    #[returns(ResponseWrapper<MetadataResponse>)]
    Metadata { token_id: u32 },
    #[returns(ResponseWrapper<Vec<Metadata>>)]
    RawMetadatas {
        start_after: Option<u32>,
        limit: Option<u8>,
    },
    #[returns(ResponseWrapper<Vec<MetadataResponse>>)]
    Metadatas {
        start_after: Option<u32>,
        limit: Option<u8>,
    },
    // MetadataLock { token_id: u32 },
}

#[cw_serde]
pub struct MetadataResponse {
    pub metadata: Metadata,
    pub metadata_id: u32,
}

#[cw_serde]
pub struct MigrateMsg {}
