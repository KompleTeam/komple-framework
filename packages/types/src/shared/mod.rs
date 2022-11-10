pub mod events;

/// Message used for instantiating a contract.
///
/// Admin is a mandatory field for every contract.
/// Data is optional and can be used to pass in instantiate data.
#[cw_serde]
pub struct RegisterMsg {
    pub admin: String,
    pub data: Option<Binary>,
}

pub const CONFIG_NAMESPACE: &str = "config";

pub const OPERATORS_NAMESPACE: &str = "operators";

pub const EXECUTE_LOCK_NAMESPACE: &str = "execute_lock";

pub const PARENT_ADDR_NAMESPACE: &str = "parent_addr";
