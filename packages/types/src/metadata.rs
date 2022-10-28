use cosmwasm_schema::cw_serde;
use std::fmt;

/// The different types of metadata.
///
/// Currently only standard, shared and dynamic metadatas are supported.
#[cw_serde]
pub enum Metadata {
    Standard,
    Shared,
    Dynamic,
}

impl Metadata {
    pub fn as_str(&self) -> &'static str {
        match self {
            Metadata::Standard => "standard",
            Metadata::Shared => "shared",
            Metadata::Dynamic => "dynamic",
        }
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Metadata::Standard => write!(f, "standard"),
            Metadata::Shared => write!(f, "shared"),
            Metadata::Dynamic => write!(f, "dynamic"),
        }
    }
}

pub const METADATA_NAMESPACE: &str = "metadata";

pub const METADATA_ID_NAMESPACE: &str = "metadata_id";

pub const LINKED_METADATA_NAMESPACE: &str = "linked_metadata";

pub const DYNAMIC_LINKED_METADATA_NAMESPACE: &str = "dynamic_linked_metadata";

pub const COLLECTION_ADDR_NAMESPACE: &str = "collection_addr";
