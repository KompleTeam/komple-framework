use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use komple_types::module::Modules;

use crate::state::{CollectionInfo, WebsiteConfig};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub name: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
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
    // Updates the general collection info
    UpdateCollectionInfo {
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
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Gets both general and website config
    Config {},
    ModuleAddress(Modules),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub admin: String,
    pub collection_info: CollectionInfo,
    pub website_config: Option<WebsiteConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
