use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum SharedExecuteMsg {
    /// Hub message.
    ///
    /// Lock the execute entry point.
    /// Can only be called by the hub module.
    LockExecute {},
}
