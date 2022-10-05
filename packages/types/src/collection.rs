use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Collections {
    Normal,
    Linked,
}

impl Collections {
    pub fn as_str(&self) -> &'static str {
        match self {
            Collections::Normal => "normal",
            Collections::Linked => "linked",
        }
    }
}

pub const COLLECTION_ADDRS_NAMESPACE: &str = "collection_addrs";

pub const LINKED_COLLECTIONS_NAMESPACE: &str = "linked_collections";

pub const COLLECTION_ID_NAMESPACE: &str = "collection_id";

pub const COLLECTION_TYPES_NAMESPACE: &str = "collection_types";
