use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::module::Modules;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ControllerQueryMsg {
    ModuleAddress(Modules),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AddressResponse {
    pub address: String,
}
