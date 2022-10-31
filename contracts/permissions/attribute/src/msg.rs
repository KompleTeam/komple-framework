use crate::state::Config;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;
use komple_types::{permission::AttributeConditions, query::ResponseWrapper};

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
pub struct AttributeMsg {
    pub collection_id: u32,
    pub token_id: u32,
    pub condition: AttributeConditions,
    pub trait_type: String,
    pub value: String,
}

#[cw_serde]
pub enum AttributeTypes {
    String,
    Integer,
    Boolean,
}
