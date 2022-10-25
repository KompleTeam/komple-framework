use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;

#[cw_serde]
pub struct RegisterMsg {
    pub admin: String,
    pub data: Option<Binary>,
}

pub const CONFIG_NAMESPACE: &str = "config";

pub const HUB_ADDR_NAMESPACE: &str = "hub_addr";

pub const OPERATORS_NAMESPACE: &str = "operators";
