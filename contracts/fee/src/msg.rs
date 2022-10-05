use crate::state::Config;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Decimal, Uint128};
use komple_types::{fee::Fees, query::ResponseWrapper};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    SetFee {
        fee_type: Fees,
        module_name: String,
        fee_name: String,
        data: Binary,
    },
    RemoveFee {
        fee_type: Fees,
        module_name: String,
        fee_name: String,
    },
    Distribute {
        fee_type: Fees,
        module_name: String,
        custom_payment_addresses: Option<Vec<CustomPaymentAddress>>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ResponseWrapper<Config>)]
    Config {},
    #[returns(ResponseWrapper<PercentageFeeResponse>)]
    PercentageFee {
        module_name: String,
        fee_name: String,
    },
    #[returns(ResponseWrapper<FixedFeeResponse>)]
    FixedFee {
        module_name: String,
        fee_name: String,
    },
    #[returns(ResponseWrapper<Vec<PercentageFeeResponse>>)]
    PercentageFees {
        module_name: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(ResponseWrapper<Vec<FixedFeeResponse>>)]
    FixedFees {
        module_name: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(ResponseWrapper<Decimal>)]
    TotalPercentageFees { module_name: String },
    #[returns(ResponseWrapper<Uint128>)]
    TotalFixedFees { module_name: String },
    #[returns(ResponseWrapper<Vec<String>>)]
    Modules {
        fee_type: Fees,
        // start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct PercentageFeeResponse {
    pub module_name: String,
    pub fee_name: String,
    pub address: Option<String>,
    pub value: Decimal,
}

#[cw_serde]
pub struct FixedFeeResponse {
    pub module_name: String,
    pub fee_name: String,
    pub address: Option<String>,
    pub value: Uint128,
}

#[cw_serde]
pub struct CustomPaymentAddress {
    pub fee_name: String,
    pub address: String,
}
