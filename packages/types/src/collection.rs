use cosmwasm_schema::cw_serde;
use std::fmt;

#[cw_serde]
pub enum Collections {
    Standard,
    // TODO: Find a new collection type
    Linked,
    Komple
}

impl Collections {
    pub fn as_str(&self) -> &'static str {
        match self {
            Collections::Standard => "standard",
            Collections::Linked => "linked",
            Collections::Komple => "komple"
        }
    }
}

impl fmt::Display for Collections {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Collections::Standard => write!(f, "standard"),
            Collections::Linked => write!(f, "linked"),
            Collections::Komple => write!(f, "komple")
        }
    }
}

pub const COLLECTION_ADDRS_NAMESPACE: &str = "collection_addrs";

pub const LINKED_COLLECTIONS_NAMESPACE: &str = "linked_collections";

pub const COLLECTION_ID_NAMESPACE: &str = "collection_id";

pub const BLACKLIST_COLLECTION_ADDRS_NAMESPACE: &str = "blacklist_collection_addrs";

pub const COLLECTION_INFO_NAMESPACE: &str = "collection_info";
