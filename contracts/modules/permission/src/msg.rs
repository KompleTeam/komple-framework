use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;
use komple_types::{execute::SharedExecuteMsg, module::Modules, query::ResponseWrapper};

#[cw_serde]
pub enum ExecuteMsg {
    RegisterPermission {
        code_id: u64,
        permission: String,
        msg: Option<Binary>,
    },
    UpdateModulePermissions {
        module: String,
        permissions: Vec<String>,
    },
    UpdateOperators {
        addrs: Vec<String>,
    },
    Check {
        module: String,
        msg: Binary,
    },
    /// Hub message.
    ///
    /// Lock the execute entry point.
    /// Can only be called by the hub module.
    LockExecute {},
}

impl From<ExecuteMsg> for SharedExecuteMsg {
    fn from(msg: ExecuteMsg) -> Self {
        match msg {
            ExecuteMsg::LockExecute {} => SharedExecuteMsg::LockExecute {},
            _ => unreachable!("Cannot convert {:?} to SharedExecuteMessage", msg),
        }
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ResponseWrapper<String>)]
    PermissionAddress { permission: String },
    #[returns(ResponseWrapper<Vec<String>>)]
    ModulePermissions(Modules),
    #[returns(ResponseWrapper<Vec<String>>)]
    Operators {},
}

#[cw_serde]
pub struct PermissionCheckMsg {
    pub permission_type: String,
    pub data: Binary,
}

#[cw_serde]
pub struct MigrateMsg {}
