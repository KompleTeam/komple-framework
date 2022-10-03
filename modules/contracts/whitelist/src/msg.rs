use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Timestamp, Uint128};
use komple_types::query::ResponseWrapper;

#[cw_serde]
pub struct InstantiateMsg {
    pub members: Vec<String>,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub unit_price: Uint128,
    pub per_address_limit: u8,
    pub member_limit: u16,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateStartTime(Timestamp),
    UpdateEndTime(Timestamp),
    AddMembers(Vec<String>),
    RemoveMembers(Vec<String>),
    UpdatePerAddressLimit(u8),
    UpdateMemberLimit(u16),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ResponseWrapper<ConfigResponse>)]
    Config {},
    #[returns(ResponseWrapper<bool>)]
    HasStarted {},
    #[returns(ResponseWrapper<bool>)]
    HasEnded {},
    #[returns(ResponseWrapper<bool>)]
    IsActive {},
    #[returns(ResponseWrapper<Vec<String>>)]
    Members {
        start_after: Option<String>,
        limit: Option<u8>,
    },
    #[returns(ResponseWrapper<bool>)]
    HasMember { member: String },
}

#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub unit_price: Uint128,
    pub per_address_limit: u8,
    pub member_limit: u16,
    pub member_num: u16,
    pub is_active: bool,
}

#[cw_serde]
pub struct MigrateMsg {}
