use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Timestamp;
use komple_types::query::ResponseWrapper;

use crate::state::WhitelistConfig;

#[cw_serde]
pub struct InstantiateMsg {
    pub members: Vec<String>,
    pub config: WhitelistConfig,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateWhitelistConfig { whitelist_config: WhitelistConfig },
    AddMembers { members: Vec<String> },
    RemoveMembers { members: Vec<String> },
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
    pub per_address_limit: u8,
    pub member_limit: u16,
    pub member_num: u16,
    pub is_active: bool,
}

#[cw_serde]
pub struct MigrateMsg {}
