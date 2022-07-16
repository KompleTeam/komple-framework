use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Metadata {
    OneToOne,
    Static,
    Dynamic,
}

impl Metadata {
    pub fn as_str(&self) -> &'static str {
        match self {
            Metadata::OneToOne => "one_to_one",
            Metadata::Static => "static",
            Metadata::Dynamic => "dynamic",
        }
    }
}
