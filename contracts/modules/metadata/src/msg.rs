use crate::state::{Config, MetaInfo, Metadata, Trait};
use cosmwasm_schema::{cw_serde, QueryResponses};
use komple_framework_types::modules::metadata::Metadata as MetadataType;
use komple_framework_types::shared::query::ResponseWrapper;

/// Message to be sent along the `RegisterMsg` for instantiation.
#[cw_serde]
pub struct InstantiateMsg {
    pub metadata_type: MetadataType,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Admin message.
    ///
    /// Add token metadata information.
    AddMetadata {
        meta_info: MetaInfo,
        attributes: Vec<Trait>,
    },
    /// Admin message.
    ///
    /// Link a token id to a metadata id.
    /// If metadata id is `None`, then the token id will be used as metadata id.
    LinkMetadata {
        token_id: u32,
        metadata_id: Option<u32>,
    },
    /// Admin message.
    ///
    /// Unlink metadata from a token id.
    UnlinkMetadata { token_id: u32 },
    /// Admin message.
    ///
    /// Update the meta info for a metadata.
    /// Can be called for raw metadata id and linked metadata id.
    UpdateMetaInfo {
        raw_metadata: bool,
        id: u32,
        meta_info: MetaInfo,
    },
    /// Admin message.
    ///
    /// Add an attribute for a metadata.
    /// Can be called for raw metadata id and linked metadata id.
    AddAttribute {
        raw_metadata: bool,
        id: u32,
        attribute: Trait,
    },
    /// Admin message.
    ///
    /// Update an attribute for a metadata.
    /// Can be called for raw metadata id and linked metadata id.
    UpdateAttribute {
        raw_metadata: bool,
        id: u32,
        attribute: Trait,
    },
    /// Admin message.
    ///
    /// Remove a trait from a metadata's attributes.
    /// Can be called for raw metadata id and linked metadata id.
    RemoveAttribute {
        raw_metadata: bool,
        id: u32,
        trait_type: String,
    },
    /// Admin message.
    ///
    /// Update the operators of this contract.
    UpdateOperators { addrs: Vec<String> },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get the contract's config.
    #[returns(ResponseWrapper<Config>)]
    Config {},
    /// Get a metadata with given raw metadata id.
    #[returns(ResponseWrapper<Metadata>)]
    RawMetadata { metadata_id: u32 },
    /// Get a metadata with given token id.
    #[returns(ResponseWrapper<MetadataResponse>)]
    Metadata { token_id: u32 },
    /// List all the raw metadata with pagination.
    #[returns(ResponseWrapper<Vec<MetadataResponse>>)]
    RawMetadatas {
        start_after: Option<u32>,
        limit: Option<u8>,
    },
    /// List all the linked metadata with pagination.
    #[returns(ResponseWrapper<Vec<MetadataResponse>>)]
    Metadatas {
        start_after: Option<u32>,
        limit: Option<u8>,
    },
    /// Get the operators of this contract.
    #[returns(ResponseWrapper<Vec<String>>)]
    Operators {},
}

#[cw_serde]
pub struct MetadataResponse {
    pub metadata: Metadata,
    pub metadata_id: u32,
}

#[cw_serde]
pub struct MigrateMsg {}
