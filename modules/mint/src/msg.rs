use cosmwasm_std::{Binary, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use token_contract::{msg::TokenInfo, state::CollectionInfo};

use rift_types::{collection::Collections, query::MintModuleQueryMsg};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateCollection {
        code_id: u64,
        collection_info: CollectionInfo,
        token_info: TokenInfo,
        per_address_limit: Option<u32>,
        start_time: Option<Timestamp>,
        whitelist: Option<String>,
        royalty: Option<String>,
        linked_collections: Option<Vec<u32>>,
    },
    UpdateMintLock {
        lock: bool,
    },
    Mint {
        collection_id: u32,
    },
    MintTo {
        collection_id: u32,
        recipient: String,
    },
    PermissionMint {
        permission_msg: Binary,
        collection_ids: Vec<u32>,
    },
    UpdateWhitelistAddresses {
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
    WhitelistAddresses {},
    CollectionTypes(Collections),
    LinkedCollections { collection_id: u32 },
}

impl From<MintModuleQueryMsg> for QueryMsg {
    fn from(msg: MintModuleQueryMsg) -> QueryMsg {
        match msg {
            MintModuleQueryMsg::CollectionAddress(collection_id) => {
                QueryMsg::CollectionAddress(collection_id)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MintMsg {
    pub collection_id: u32,
    pub owner: String,
}
