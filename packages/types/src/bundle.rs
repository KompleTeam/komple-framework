use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Bundles {
    Normal,
    Linked,
}

impl Bundles {
    pub fn as_str(&self) -> &'static str {
        match self {
            Bundles::Normal => "normal",
            Bundles::Linked => "linked",
        }
    }
}

pub const BUNDLE_ADDRS_NAMESPACE: &str = "bundle_addrs";

pub const LINKED_BUNDLES_NAMESPACE: &str = "linked_bundles";

pub const BUNDLE_ID_NAMESPACE: &str = "bundle_id";

pub const BUNDLE_TYPES_NAMESPACE: &str = "bundle_types";
