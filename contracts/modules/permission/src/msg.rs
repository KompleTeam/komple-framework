use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;
use komple_types::shared::execute::SharedExecuteMsg;
use komple_types::shared::query::ResponseWrapper;

#[cw_serde]
pub enum ExecuteMsg {
    /// Admin message.
    ///
    /// Register a new permission to the module.
    /// Saves the permission address to storage.
    RegisterPermission {
        code_id: u64,
        permission: String,
        msg: Option<Binary>,
    },
    /// Admin message.
    ///
    /// Update the permissions for a module.
    /// Permissions must be set for usage.
    UpdateModulePermissions {
        module: String,
        permissions: Vec<String>,
    },
    /// Admin message.
    ///
    /// Updates the operators of this contract.
    UpdateOperators { addrs: Vec<String> },
    /// Public message.
    ///
    /// Entry point for permission messages.
    /// Permission messages are constructed and sent in this message.
    Check { module: String, msg: Binary },
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
    /// Resolve the permission address for a permission.
    #[returns(ResponseWrapper<String>)]
    PermissionAddress { permission: String },
    /// List all the permissions for a module.
    #[returns(ResponseWrapper<Vec<String>>)]
    ModulePermissions { module: String },
    /// Get the operators of this contract.
    #[returns(ResponseWrapper<Vec<String>>)]
    Operators {},
}

/// Message used for permission check messages.
#[cw_serde]
pub struct PermissionCheckMsg {
    pub permission_type: String,
    pub data: Binary,
}

#[cw_serde]
pub struct MigrateMsg {}
