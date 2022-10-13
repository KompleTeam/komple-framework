use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ResponseWrapper<T> {
    pub query: String,
    pub data: T,
}
impl<T> ResponseWrapper<T> {
    pub fn new(query: &str, data: T) -> Self {
        Self {
            query: query.to_string(),
            data,
        }
    }
}
