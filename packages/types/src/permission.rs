use cosmwasm_schema::cw_serde;
use std::fmt;

/// The different types of permissions.
///
/// Currently only ownership, attribute and link are supported.
#[cw_serde]
pub enum Permissions {
    Ownership,
    Attribute,
    Link,
}
impl Permissions {
    pub fn as_str(&self) -> &str {
        match self {
            Permissions::Ownership => "ownership",
            Permissions::Attribute => "attribute",
            Permissions::Link => "link",
        }
    }
}
impl fmt::Display for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Permissions::Ownership => write!(f, "ownership"),
            Permissions::Attribute => write!(f, "attribute"),
            Permissions::Link => write!(f, "link"),
        }
    }
}

/// The different types of attribute permission conditions.
#[cw_serde]
pub enum AttributeConditions {
    Exist,
    Absent,
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}
impl AttributeConditions {
    pub fn as_str(&self) -> &str {
        match self {
            AttributeConditions::Exist => "exist",
            AttributeConditions::Absent => "absent",
            AttributeConditions::Equal => "equal",
            AttributeConditions::NotEqual => "not_equal",
            AttributeConditions::GreaterThan => "greater_than",
            AttributeConditions::GreaterThanOrEqual => "greater_than_or_equal",
            AttributeConditions::LessThan => "less_than",
            AttributeConditions::LessThanOrEqual => "less_than_or_equal",
        }
    }
}

pub const MODULE_PERMISSIONS_NAMESPACE: &str = "module_permissions";

pub const PERMISSION_MODULE_ADDR_NAMESPACE: &str = "permission_module_addr";

pub const PERMISSION_ID_NAMESPACE: &str = "permission_id";

pub const PERMISSION_TO_REGISTER_NAMESPACE: &str = "permission_to_register";

pub const PERMISSIONS_NAMESPACE: &str = "permissions";
