use crate::state::{HubInfo, WebsiteConfig};
use cosmwasm_schema::{cw_serde, QueryResponses};
use komple_types::{module::Modules, query::ResponseWrapper};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Instantiates a new mint module
    InitMintModule {
        code_id: u64,
    },
    // Instantiates a new permission module
    InitPermissionModule {
        code_id: u64,
    },
    // Instantiates a new merge module
    InitMergeModule {
        code_id: u64,
    },
    // Instantiates a new marketplace module
    InitMarketplaceModule {
        code_id: u64,
        native_denom: String,
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
    RemoveNativeModule {
        module: Modules,
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
    ModuleAddress(Modules),
    #[returns(ResponseWrapper<Vec<String>>)]
    Operators {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub admin: String,
    pub hub_info: HubInfo,
    pub website_config: Option<WebsiteConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
