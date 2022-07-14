use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Collections {
    Normal,
    // Multiple,
    Linked,
    // OneToOne,
}

impl Collections {
    pub fn as_str(&self) -> &'static str {
        match self {
            Collections::Normal => "normal",
            // Collections::Multiple => "multiple",
            Collections::Linked => "linked",
            // Collections::OneToOne => "one_to_one",
        }
    }
}
