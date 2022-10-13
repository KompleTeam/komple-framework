use crate::state::{CollectionConfig, CollectionInfo, SubModules};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Timestamp, Uint128};
use cw721::CustomMsg;
use komple_metadata_module::msg::InstantiateMsg as MetadataInstantiateMsg;
use komple_types::{query::ResponseWrapper, tokens::Locks};
use komple_whitelist_module::msg::InstantiateMsg as WhitelistInstantiateMsg;

#[cw_serde]
pub struct TokenInfo {
    pub symbol: String,
    pub minter: String,
}

#[cw_serde]
pub struct MetadataInfo {
    pub instantiate_msg: MetadataInstantiateMsg,
    pub code_id: u64,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub creator: String,
    pub token_info: TokenInfo,
    pub collection_info: CollectionInfo,
    pub collection_config: CollectionConfig,
    pub metadata_info: MetadataInfo,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Reimplementation of cw721 messages
    TransferNft {
        recipient: String,
        token_id: String,
    },
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },
    Mint {
        owner: String,
        metadata_id: Option<u32>,
    },
    Burn {
        token_id: String,
    },

    // ADMIN MESSAGES
    UpdateModuleOperators {
        addrs: Vec<String>,
    },
    AdminTransferNft {
        recipient: String,
        token_id: String,
    },

    // LOCK MESSAGES
    UpdateLocks {
        locks: Locks,
    },
    UpdateTokenLocks {
        token_id: String,
        locks: Locks,
    },

    // CONFIG MESSAGES
    UpdatePerAddressLimit {
        per_address_limit: Option<u32>,
    },
    UpdateStartTime {
        start_time: Option<Timestamp>,
    },

    // CONTRACT MESSAGES
    InitWhitelistContract {
        code_id: u64,
        instantiate_msg: WhitelistInstantiateMsg,
    },
}
impl CustomMsg for ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Custom query messages
    #[returns(ResponseWrapper<Locks>)]
    Locks {},
    #[returns(ResponseWrapper<Locks>)]
    TokenLocks { token_id: String },
    #[returns(ResponseWrapper<u32>)]
    MintedTokensPerAddress { address: String },
    #[returns(ResponseWrapper<CollectionInfo>)]
    CollectionInfo {},
    #[returns(ResponseWrapper<SubModules>)]
    SubModules {},
    #[returns(ResponseWrapper<ConfigResponse>)]
    Config {},
    #[returns(ResponseWrapper<Vec<String>>)]
    ModuleOperators {},
}
impl CustomMsg for QueryMsg {}

#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub creator: String,
    pub native_denom: String,
    pub per_address_limit: Option<u32>,
    pub start_time: Option<Timestamp>,
    pub max_token_limit: Option<u32>,
    pub unit_price: Option<Uint128>,
}

#[cw_serde]
pub struct LocksReponse {
    pub locks: Locks,
}

#[cw_serde]
pub struct MintedTokenAmountResponse {
    pub amount: u32,
}

#[cw_serde]
pub struct MigrateMsg {}
