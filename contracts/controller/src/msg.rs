use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use komple_types::module::Modules;

use crate::state::{ControllerInfo, WebsiteConfig};

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
    InitMintModule {
        code_id: u64,
    },
    InitPermissionModule {
        code_id: u64,
    },
    InitMergeModule {
        code_id: u64,
    },
    InitMarketplaceModule {
        code_id: u64,
        native_denom: String,
    },
    UpdateWebsiteConfig {
        background_color: Option<String>,
        background_image: Option<String>,
        banner_image: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    ContollerInfo {},
    ModuleAddress(Modules),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub admin: String,
    pub controller_info: ControllerInfo,
    pub website_config: Option<WebsiteConfig>,
}
