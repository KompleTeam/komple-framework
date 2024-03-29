use cosmwasm_schema::cw_serde;
use std::fmt;

/// The different types of collections.
///
/// Currently only standard and komple collections are supported.
#[cw_serde]
pub enum Collections {
    Standard,
    Komple,
}

impl Collections {
    pub fn as_str(&self) -> &'static str {
        match self {
            Collections::Standard => "standard",
            Collections::Komple => "komple",
        }
    }
}

impl fmt::Display for Collections {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Collections::Standard => write!(f, "standard"),
            Collections::Komple => write!(f, "komple"),
        }
    }
}

pub const COLLECTION_ADDRS_NAMESPACE: &str = "collection_addrs";

pub const LINKED_COLLECTIONS_NAMESPACE: &str = "linked_collections";

pub const COLLECTION_ID_NAMESPACE: &str = "collection_id";

pub const BLACKLIST_COLLECTION_ADDRS_NAMESPACE: &str = "blacklist_collection_addrs";

pub const COLLECTION_INFO_NAMESPACE: &str = "collection_info";

pub const CREATORS_NAMESPACE: &str = "creators";

pub const MINT_LOCKS_NAMESPACE: &str = "mint_locks";
