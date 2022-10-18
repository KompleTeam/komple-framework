use crate::state::{CollectionConfig, Config};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Timestamp};
use cw721::CustomMsg;
use komple_metadata_module::msg::InstantiateMsg as MetadataInstantiateMsg;
use komple_types::{collection::Collections, query::ResponseWrapper, token::{Locks, SubModules}};
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
    pub collection_name: String,
    pub collection_type: Collections,
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
    #[returns(ResponseWrapper<SubModules>)]
    SubModules {},
    #[returns(ResponseWrapper<Config>)]
    Config {},
    #[returns(ResponseWrapper<Vec<String>>)]
    ModuleOperators {},
}
impl CustomMsg for QueryMsg {}

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
