use crate::state::Config;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use komple_types::shared::query::ResponseWrapper;
use komple_types::modules::fee::Fees;
use komple_types::shared::execute::SharedExecuteMsg;

#[cw_serde]
pub enum ExecuteMsg {
    /// Admin message.
    ///
    /// Creates a new fee configuration.
    /// Fees are tied to a module with a fee name.
    SetFee {
        fee_type: Fees,
        module_name: String,
        fee_name: String,
        data: Binary,
    },
    /// Admin message.
    ///
    /// Removes a fee configuration.
    RemoveFee {
        fee_type: Fees,
        module_name: String,
        fee_name: String,
    },
    /// Public message.
    ///
    /// Distributes the sent funds according to the fee configuration.
    /// Custom payment addresses can be specified for
    /// overriding the default payment addresses.
    Distribute {
        fee_type: Fees,
        module_name: String,
        custom_payment_addresses: Option<Vec<CustomPaymentAddress>>,
    },
    /// Hub message.
    ///
    /// Lock the execute entry point.
    /// Can only be called by the hub module.
    LockExecute {},
    Receive(Cw20ReceiveMsg),
}

impl From<ExecuteMsg> for SharedExecuteMsg {
    fn from(msg: ExecuteMsg) -> Self {
        match msg {
            ExecuteMsg::LockExecute {} => SharedExecuteMsg::LockExecute {},
            _ => unreachable!("Cannot convert {:?} to SharedExecuteMessage", msg),
        }
    }
}

#[cw_serde]
pub enum ReceiveMsg {
    Distribute {
        fee_type: Fees,
        module_name: String,
        custom_payment_addresses: Option<Vec<CustomPaymentAddress>>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Gets the contract's config.
    #[returns(ResponseWrapper<Config>)]
    Config {},
    /// Gets the fee configuration for a module and fee name. Used for percentage fees.
    #[returns(ResponseWrapper<PercentageFeeResponse>)]
    PercentageFee {
        module_name: String,
        fee_name: String,
    },
    /// Gets the fee configuration for a module and fee name. Used for fixed fees.
    #[returns(ResponseWrapper<FixedFeeResponse>)]
    FixedFee {
        module_name: String,
        fee_name: String,
    },
    /// Gets the fee configurations for a module with pagination. Used for percentage fees.
    #[returns(ResponseWrapper<Vec<PercentageFeeResponse>>)]
    PercentageFees {
        module_name: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Gets the fee configurations for a module with pagination. Used for fixed fees.
    #[returns(ResponseWrapper<Vec<FixedFeeResponse>>)]
    FixedFees {
        module_name: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Gets the sum of all the percentages for a given module.
    #[returns(ResponseWrapper<Decimal>)]
    TotalPercentageFees {
        module_name: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Gets the sum of all the fixed amounts for a given module.
    #[returns(ResponseWrapper<Uint128>)]
    TotalFixedFees {
        module_name: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Gets all the module names and fee names for a given fee type.
    #[returns(ResponseWrapper<Vec<String>>)]
    Keys {
        fee_type: Fees,
        start_after: Option<String>,
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

/// Used for overriding the default payment addresses.
#[cw_serde]
pub struct CustomPaymentAddress {
    pub fee_name: String,
    pub address: String,
}
