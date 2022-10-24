use crate::state::{HubInfo, WebsiteConfig};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;
use komple_types::query::ResponseWrapper;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
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
    // Updates the website profile configuration
    UpdateWebsiteConfig {
        background_color: Option<String>,
        background_image: Option<String>,
        banner_image: Option<String>,
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
    // Gets both general and website config
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
    pub website_config: Option<WebsiteConfig>,
}

#[cw_serde]
pub struct MigrateMsg {}
