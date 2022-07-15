use cosmwasm_std::Binary;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use rift_types::{module::Modules, permission::Permissions};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateModulePermissions {
        module: Modules,
        permissions: Vec<Permissions>,
    },
    UpdateWhitelistAddresses {
        addrs: Vec<String>,
    },
    Check {
        module: Modules,
        msg: Binary,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    ModulePermissions(Modules),
    WhitelistAddresses {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct OwnershipMsg {
    pub collection_id: u32,
    pub token_id: u32,
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PermissionCheckMsg {
    pub permission_type: Permissions,
    pub data: Binary,
}
