use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ModuleInstantiateMsg {
    pub admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MarketplaceModuleInstantiateMsg {
    pub admin: String,
    pub native_denom: String,
}
