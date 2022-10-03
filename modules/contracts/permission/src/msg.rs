use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;
use komple_types::{module::Modules, permission::Permissions, query::ResponseWrapper};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateModulePermissions {
        module: Modules,
        permissions: Vec<Permissions>,
    },
    UpdateOperators {
        addrs: Vec<String>,
    },
    Check {
        module: Modules,
        msg: Binary,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ResponseWrapper<Vec<String>>)]
    ModulePermissions(Modules),
    #[returns(ResponseWrapper<Vec<String>>)]
    Operators {},
}

#[cw_serde]
pub struct OwnershipMsg {
    pub collection_id: u32,
    pub token_id: u32,
    pub owner: String,
}

#[cw_serde]
pub struct PermissionCheckMsg {
    pub permission_type: Permissions,
    pub data: Binary,
}

#[cw_serde]
pub struct MigrateMsg {}
