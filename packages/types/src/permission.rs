use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Permissions {
    Ownership,
    Attribute,
}

impl Permissions {
    pub fn as_str(&self) -> &str {
        match self {
            Permissions::Ownership => "ownership",
            Permissions::Attribute => "attribute",
        }
    }
}
