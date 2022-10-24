use crate::state::Config;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;
use komple_types::query::ResponseWrapper;

#[cw_serde]
pub enum ExecuteMsg {
    Check { data: Binary },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ResponseWrapper<Config>)]
    Config {},
}

#[cw_serde]
pub struct OwnershipMsg {
    pub collection_id: u32,
    pub token_id: u32,
    pub address: String,
}
