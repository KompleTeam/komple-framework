use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::Config;

use komple_framework_types::shared::{execute::SharedExecuteMsg, query::ResponseWrapper};

#[cw_serde]
pub struct InstantiateMsg {
    /* TODO: Add instantiate fields here */
    /* ... */
}

#[cw_serde]
pub enum ExecuteMsg {
    /* TODO: Add execute messages here */
    /* ... */
    
    /// Admin message.
    ///
    /// Update the operators of this contract.
    UpdateOperators { addrs: Vec<String> },
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
    /* TODO: Add query messages here */
    /* ... */

    /// Get the contract's config.
    #[returns(ResponseWrapper<Config>)]
    Config {},
    /// Get the operators of this contract.
    #[returns(ResponseWrapper<Vec<String>>)]
    Operators {},
}

#[cw_serde]
pub struct MigrateMsg {}
