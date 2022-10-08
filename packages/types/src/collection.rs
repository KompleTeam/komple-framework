use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Collections {
    Standard,
    // TODO: Find a new collection type
    Linked,
}

impl Collections {
    pub fn as_str(&self) -> &'static str {
        match self {
            Collections::Standard => "standard",
            Collections::Linked => "linked",
        }
    }
}

pub const COLLECTION_ADDRS_NAMESPACE: &str = "collection_addrs";

pub const LINKED_COLLECTIONS_NAMESPACE: &str = "linked_collections";

pub const COLLECTION_ID_NAMESPACE: &str = "collection_id";

pub const COLLECTION_TYPES_NAMESPACE: &str = "collection_types";

pub const BLACKLIST_COLLECTION_ADDRS_NAMESPACE: &str = "blacklist_collection_addrs";
