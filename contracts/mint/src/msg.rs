use cosmwasm_std::Binary;
use komple_token_module::msg::InstantiateMsg as TokenInstantiateMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use komple_types::bundle::Bundles;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateBundle {
        code_id: u64,
        token_instantiate_msg: TokenInstantiateMsg,
        linked_bundles: Option<Vec<u32>>,
    },
    UpdatePublicBundleCreation {
        public_bundle_creation: bool,
    },
    UpdateMintLock {
        lock: bool,
    },
    Mint {
        bundle_id: u32,
        metadata_id: Option<u32>,
    },
    MintTo {
        bundle_id: u32,
        recipient: String,
        metadata_id: Option<u32>,
    },
    PermissionMint {
        permission_msg: Binary,
        bundle_ids: Vec<u32>,
        metadata_ids: Option<Vec<u32>>,
    },
    UpdateOperators {
        addrs: Vec<String>,
    },
    UpdateLinkedBundles {
        bundle_id: u32,
        linked_bundles: Vec<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    BundleAddress(u32),
    Operators {},
    BundleTypes(Bundles),
    LinkedBundles {
        bundle_id: u32,
    },
    Bundles {
        start_after: Option<u32>,
        limit: Option<u8>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MintMsg {
    pub bundle_id: u32,
    pub owner: String,
    pub metadata_id: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BundlesResponse {
    pub bundle_id: u32,
    pub address: String,
}
