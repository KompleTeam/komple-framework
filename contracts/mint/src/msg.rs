use cosmwasm_std::Binary;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use token_contract::msg::InstantiateMsg as TokenInstantiateMsg;

use komple_types::collection::Collections;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateCollection {
        code_id: u64,
        token_instantiate_msg: TokenInstantiateMsg,
        linked_collections: Option<Vec<u32>>,
    },
    UpdateMintLock {
        lock: bool,
    },
    Mint {
        collection_id: u32,
        metadata_id: Option<u32>,
    },
    MintTo {
        collection_id: u32,
        recipient: String,
        metadata_id: Option<u32>,
    },
    PermissionMint {
        permission_msg: Binary,
        collection_ids: Vec<u32>,
        metadata_ids: Option<Vec<u32>>,
    },
    UpdateOperators {
        addrs: Vec<String>,
    },
    UpdateLinkedCollections {
        collection_id: u32,
        linked_collections: Vec<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    CollectionAddress(u32),
    Operators {},
    CollectionTypes(Collections),
    LinkedCollections { collection_id: u32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MintMsg {
    pub collection_id: u32,
    pub owner: String,
    pub metadata_id: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
