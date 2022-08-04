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
pub enum MintModuleQueryMsg {
    CollectionAddress(u32),
    LinkedCollections { collection_id: u32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenContractQueryMsg {
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AddressResponse {
    pub address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MultipleAddressResponse {
    pub addresses: Vec<String>,
}
