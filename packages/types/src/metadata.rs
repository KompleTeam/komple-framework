use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
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

pub const METADATA_NAMESPACE: &str = "metadata";

pub const METADATA_ID_NAMESPACE: &str = "metadata_id";

pub const LINKED_METADATA_NAMESPACE: &str = "linked_metadata";

pub const DYNAMIC_LINKED_METADATA_NAMESPACE: &str = "dynamic_linked_metadata";

pub const BUNDLE_ADDR_NAMESPACE: &str = "bundle_addr";
