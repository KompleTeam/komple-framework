use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Royalty {
    Admin,
    Owners,
    Tokens,
}

impl Royalty {
    pub fn as_str(&self) -> &'static str {
        match self {
            Royalty::Admin => "admin",
            Royalty::Owners => "owners",
            Royalty::Tokens => "tokens",
        }
    }
}
