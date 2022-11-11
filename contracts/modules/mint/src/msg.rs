use crate::state::{CollectionInfo, Config};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;
use cw20::Cw20ReceiveMsg;
use komple_token_module::{
    msg::{MetadataInfo, TokenInfo},
    state::CollectionConfig,
};
use komple_types::shared::execute::SharedExecuteMsg;
use komple_types::shared::query::ResponseWrapper;

#[cw_serde]
pub struct CollectionFundInfo {
    pub is_native: bool,
    pub denom: String,
    pub cw20_address: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Admin and public message.
    ///
    /// Create a new collection.
    /// This can be executed by both admin and users based on configuration.
    CreateCollection {
        code_id: u64,
        collection_info: CollectionInfo,
        collection_config: CollectionConfig,
        token_info: TokenInfo,
        metadata_info: MetadataInfo,
        fund_info: CollectionFundInfo,
        linked_collections: Option<Vec<u32>>,
    },
    /// Admin message.
    ///
    /// Update the configuration for public collection creation.
    /// If set to true, users can create collections.
    UpdatePublicCollectionCreation {
        public_collection_creation: bool,
    },
    /// Admin message.
    ///
    /// Update the configuration for collection mint lock.
    UpdateCollectionMintLock {
        collection_id: u32,
        lock: bool,
    },
    /// Public message.
    ///
    /// Mint a new token on a collection.
    /// Additional metadata id can be provided to link to a certain metadata.
    Mint {
        collection_id: u32,
        metadata_id: Option<u32>,
    },
    /// Admin message.
    ///
    /// Same as `Mint` message but only executable by admin.
    AdminMint {
        collection_id: u32,
        recipient: String,
        metadata_id: Option<u32>,
    },
    /// Admin message.
    ///
    /// Same as `Mint` message but can be used with permissions.
    PermissionMint {
        permission_msg: Binary,
        mint_msg: MintMsg,
    },
    /// Admin message.
    ///
    /// Update the operators of this contract.
    UpdateOperators {
        addrs: Vec<String>,
    },
    /// Admin message.
    ///
    /// Update the linked collections of a collection.
    UpdateLinkedCollections {
        collection_id: u32,
        linked_collections: Vec<u32>,
    },
    /// Admin message.
    ///
    /// Update the status of a collection. Whitelist or blacklist.
    UpdateCollectionStatus {
        collection_id: u32,
        is_blacklist: bool,
    },
    /// Hub message.
    ///
    /// Lock the execute entry point.
    /// Can only be called by the hub module.
    LockExecute {},
    /// Admin message.
    ///
    /// Update addresses that can create collections.
    UpdateCreators {
        addrs: Vec<String>,
    },
    Receive(Cw20ReceiveMsg),
}

impl From<ExecuteMsg> for SharedExecuteMsg {
    fn from(msg: ExecuteMsg) -> Self {
        match msg {
            ExecuteMsg::LockExecute {} => SharedExecuteMsg::LockExecute {},
            _ => unreachable!("Cannot convert {:?} to SharedExecuteMessage", msg),
        }
    }
}

#[cw_serde]
pub enum ReceiveMsg {
    Mint {
        collection_id: u32,
        metadata_id: Option<u32>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get the contract's config.
    #[returns(ResponseWrapper<Config>)]
    Config {},
    /// Resolve the collection address for a collection id.
    #[returns(ResponseWrapper<String>)]
    CollectionAddress(u32),
    /// Get the collection info for a collection id.
    #[returns(ResponseWrapper<CollectionInfo>)]
    CollectionInfo { collection_id: u32 },
    /// Get the operators of this contract.
    #[returns(ResponseWrapper<Vec<String>>)]
    Operators {},
    /// Get the linked collections of a collection.
    #[returns(ResponseWrapper<Vec<u32>>)]
    LinkedCollections { collection_id: u32 },
    /// List the collections with pagination.
    #[returns(ResponseWrapper<Vec<CollectionsResponse>>)]
    Collections {
        blacklist: bool,
        start_after: Option<u32>,
        limit: Option<u8>,
    },
    /// Get the creators of this contract.
    #[returns(ResponseWrapper<Vec<String>>)]
    Creators {},
    /// Get the mint lock for collection
    #[returns(ResponseWrapper<Option<bool>>)]
    MintLock { collection_id: u32 },
}

/// Message used to mint new tokens on a collection.
#[cw_serde]
pub struct MintMsg {
    pub collection_id: u32,
    pub recipient: String,
    pub metadata_id: Option<u32>,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct CollectionsResponse {
    pub collection_id: u32,
    pub address: String,
}
