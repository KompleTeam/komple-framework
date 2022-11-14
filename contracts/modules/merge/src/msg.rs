use crate::state::Config;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;
use komple_framework_types::shared::execute::SharedExecuteMsg;
use komple_framework_types::shared::query::ResponseWrapper;

#[cw_serde]
pub enum ExecuteMsg {
    /// Admin message.
    ///
    /// Update the lock for merging.
    /// This applies for the normal merge operation.
    UpdateMergeLock { lock: bool },
    /// Public message.
    ///
    /// Burn multiple tokens and mint a new one.
    /// Takes merge msg to determine which tokens to burn and which to mint.
    Merge { msg: MergeMsg },
    /// Admin message.
    ///
    /// Same as `Merge` message but can be used with permissions.
    PermissionMerge {
        permission_msg: Binary,
        merge_msg: MergeMsg,
    },
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
    /// Get the contract's config.
    #[returns(ResponseWrapper<Config>)]
    Config {},
    /// Get the operators of this contract.
    #[returns(ResponseWrapper<Vec<String>>)]
    Operators {},
}

/// Message that is used for the tokens that will be burned.
#[cw_serde]
pub struct MergeBurnMsg {
    pub collection_id: u32,
    pub token_id: u32,
}

/// Message that is used for the merge operation.
#[cw_serde]
pub struct MergeMsg {
    pub recipient: String,
    pub mint_id: u32,
    pub metadata_id: Option<u32>,
    pub burn_ids: Vec<MergeBurnMsg>,
}

#[cw_serde]
pub struct MigrateMsg {}
