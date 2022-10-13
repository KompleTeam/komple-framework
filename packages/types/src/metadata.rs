use cosmwasm_schema::cw_serde;

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
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

pub const METADATA_NAMESPACE: &str = "metadata";

pub const METADATA_ID_NAMESPACE: &str = "metadata_id";

pub const LINKED_METADATA_NAMESPACE: &str = "linked_metadata";

pub const DYNAMIC_LINKED_METADATA_NAMESPACE: &str = "dynamic_linked_metadata";

pub const COLLECTION_ADDR_NAMESPACE: &str = "collection_addr";
