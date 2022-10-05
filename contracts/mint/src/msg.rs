use crate::state::Config;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;
use komple_token_module::msg::InstantiateMsg as TokenInstantiateMsg;
use komple_types::{collection::Collections, query::ResponseWrapper};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateCollection {
        code_id: u64,
        token_instantiate_msg: TokenInstantiateMsg,
        linked_collections: Option<Vec<u32>>,
    },
    UpdatePublicCollectionCreation {
        public_collection_creation: bool,
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
    WhitelistCollection {
        collection_id: u32,
    },
    BlacklistCollection {
        collection_id: u32,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ResponseWrapper<Config>)]
    Config {},
    #[returns(ResponseWrapper<String>)]
    CollectionAddress(u32),
    #[returns(ResponseWrapper<Vec<String>>)]
    Operators {},
    #[returns(ResponseWrapper<Vec<u32>>)]
    CollectionTypes(Collections),
    #[returns(ResponseWrapper<Vec<u32>>)]
    LinkedCollections { collection_id: u32 },
    #[returns(ResponseWrapper<Vec<CollectionsResponse>>)]
    Collections {
        blacklist: bool,
        start_after: Option<u32>,
        limit: Option<u8>,
    },
}

#[cw_serde]
pub struct MintMsg {
    pub collection_id: u32,
    pub owner: String,
    pub metadata_id: Option<u32>,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct CollectionsResponse {
    pub collection_id: u32,
    pub address: String,
}
