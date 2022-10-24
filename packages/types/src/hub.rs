use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;

#[cw_serde]
pub struct RegisterMsg {
    pub admin: String,
    pub data: Option<Binary>,
}

pub const HUB_INFO_NAMESPACE: &str = "hub_info";

pub const WEBSITE_CONFIG_NAMESPACE: &str = "website_config";

pub const MODULE_ID_NAMESPACE: &str = "module_id";

pub const MODULE_TO_REGISTER_NAMESPACE: &str = "module_to_register";

pub const MARBU_FEE_MODULE_NAMESPACE: &str = "marbu_fee_module";
