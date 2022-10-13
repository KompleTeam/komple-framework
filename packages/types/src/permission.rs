use cosmwasm_schema::cw_serde;
use std::fmt;

#[cw_serde]
pub enum Permissions {
    Ownership,
    Attribute,
}

impl Permissions {
    pub fn as_str(&self) -> &str {
        match self {
            Permissions::Ownership => "ownership",
            Permissions::Attribute => "attribute",
        }
    }
}

impl fmt::Display for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Permissions::Ownership => write!(f, "ownership"),
            Permissions::Attribute => write!(f, "attribute"),
        }
    }
}

pub const MODULE_PERMISSIONS_NAMESPACE: &str = "module_permissions";

pub const PERMISSION_MODULE_ADDR_NAMESPACE: &str = "permission_module_addr";

pub const PERMISSION_ID_NAMESPACE: &str = "permission_id";

pub const PERMISSION_TO_REGISTER_NAMESPACE: &str = "permission_to_register";

pub const PERMISSIONS_NAMESPACE: &str = "permissions";
