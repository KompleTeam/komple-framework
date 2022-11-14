use crate::state::{CollectionConfig, Config};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;
use cw721::CustomMsg;
use komple_framework_metadata_module::msg::InstantiateMsg as MetadataInstantiateMsg;
use komple_framework_types::modules::mint::Collections;
use komple_framework_types::modules::token::{Locks, SubModules};
use komple_framework_types::shared::query::ResponseWrapper;
use komple_framework_whitelist_module::msg::InstantiateMsg as WhitelistInstantiateMsg;

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
    // These messages are the same with custom implementation
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
    /// Admin message.
    ///
    /// Update the operators of this contract.
    UpdateModuleOperators {
        addrs: Vec<String>,
    },
    /// Admin message.
    ///
    /// Same message as `TransferNft` but can only be used by admin.
    AdminTransferNft {
        recipient: String,
        token_id: String,
    },
    /// Admin message.
    ///
    /// Lock the module to prevent some operations.
    /// Includes minting, burning, transferring and sending.
    UpdateLocks {
        locks: Locks,
    },
    /// Admin message.
    ///
    /// Lock a single token to prevent some operations.
    /// Includes minting, burning, transferring and sending.
    UpdateTokenLocks {
        token_id: String,
        locks: Locks,
    },
    /// Admin message.
    ///
    /// Update the collection config.
    UpdateCollectionConfig {
        collection_config: CollectionConfig,
    },
    /// Admin message.
    ///
    /// Create a whitelist contract tied to this contract.
    InitWhitelistContract {
        code_id: u64,
        instantiate_msg: WhitelistInstantiateMsg,
    },
}
impl CustomMsg for ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// List operation locks for the contract.
    #[returns(ResponseWrapper<Locks>)]
    Locks {},
    /// List operation locks for a token.
    #[returns(ResponseWrapper<Locks>)]
    TokenLocks { token_id: String },
    /// Get the total amount of minted tokens for an address.
    #[returns(ResponseWrapper<u32>)]
    MintedTokensPerAddress { address: String },
    /// List the sub modules for this contract.
    #[returns(ResponseWrapper<SubModules>)]
    SubModules {},
    /// Get this contract's configuration.
    #[returns(ResponseWrapper<Config>)]
    Config {},
    /// Get the operators of this contract.
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
