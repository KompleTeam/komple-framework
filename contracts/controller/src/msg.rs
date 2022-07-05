use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use mint::msg::InstantiateMsg as MintInstantiateMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub name: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
    pub mint_code_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateMintCodeId { code_id: u64 },
    AddCollection { instantiate_msg: MintInstantiateMsg },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetCollection { collection_id: u32 },
    GetContollerInfo {},
}
