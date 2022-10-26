use crate::state::HubInfo;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;
use komple_types::query::ResponseWrapper;

#[cw_serde]
pub struct InstantiateMsg {
    pub hub_info: HubInfo,
    pub marbu_fee_module: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterModule {
        code_id: u64,
        module: String,
        msg: Option<Binary>,
    },
    // Updates the general hub info
    UpdateHubInfo {
        name: String,
        description: String,
        image: String,
        external_link: Option<String>,
    },
    DeregisterModule {
        module: String,
    },
    UpdateOperators {
        addrs: Vec<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ResponseWrapper<ConfigResponse>)]
    Config {},
    #[returns(ResponseWrapper<String>)]
    ModuleAddress { module: String },
    #[returns(ResponseWrapper<Vec<String>>)]
    Operators {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub hub_info: HubInfo,
}

#[cw_serde]
pub struct MigrateMsg {}
