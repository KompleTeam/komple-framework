use cosmwasm_schema::cw_serde;

#[cw_serde]
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
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

pub const COLLECTION_ADDRS_NAMESPACE: &str = "collection_addrs";

pub const LINKED_COLLECTIONS_NAMESPACE: &str = "linked_collections";

pub const COLLECTION_ID_NAMESPACE: &str = "collection_id";

pub const BLACKLIST_COLLECTION_ADDRS_NAMESPACE: &str = "blacklist_collection_addrs";
