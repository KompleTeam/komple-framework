use cosmwasm_std::{Binary, Decimal, Empty, Timestamp};
use cw721::Expiration;
use cw721_base::{ExecuteMsg as Cw721ExecuteMsg, QueryMsg as Cw721QueryMsg};
use rift_types::{
    metadata::Metadata as MetadataType, query::TokenContractQueryMsg, royalty::Royalty,
    tokens::Locks,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{CollectionInfo, Contracts};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokenInfo {
    pub symbol: String,
    pub minter: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub admin: String,
    pub token_info: TokenInfo,
    pub per_address_limit: Option<u32>,
    pub start_time: Option<Timestamp>,
    pub collection_info: CollectionInfo,
    pub max_token_limit: Option<u32>,
    pub contracts: Contracts,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // CW721 MESSAGES
    TransferNft {
        recipient: String,
        token_id: String,
    },
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },
    Approve {
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    },
    Revoke {
        spender: String,
        token_id: String,
    },
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    RevokeAll {
        operator: String,
    },
    Mint {
        owner: String,
    },
    Burn {
        token_id: String,
    },

    // ADMIN MESSAGES
    UpdateOperators {
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
    UpdateTokenLock {
        token_id: String,
        locks: Locks,
    },
    UpdateOperationLock {
        lock: bool,
    },

    // CONFIG MESSAGES
    UpdatePerAddressLimit {
        per_address_limit: Option<u32>,
    },
    UpdateStartTime {
        start_time: Option<Timestamp>,
    },
    UpdateWhitelist {
        whitelist: Option<String>,
    },
    UpdateRoyalty {
        royalty: Option<String>,
    },
    UpdateMetadata {
        metadata: Option<String>,
    },

    // CONTRACT MESSAGES
    InitMetadataContract {
        code_id: u64,
        metadata_type: MetadataType,
    },
    InitRoyaltyContract {
        code_id: u64,
        share: Decimal,
        royalty_type: Royalty,
    },
}

impl From<ExecuteMsg> for Cw721ExecuteMsg<Empty> {
    fn from(msg: ExecuteMsg) -> Cw721ExecuteMsg<Empty> {
        match msg {
            ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => Cw721ExecuteMsg::TransferNft {
                recipient,
                token_id,
            },
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            } => Cw721ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            },
            ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => Cw721ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            },
            ExecuteMsg::Revoke { spender, token_id } => {
                Cw721ExecuteMsg::Revoke { spender, token_id }
            }
            ExecuteMsg::ApproveAll { operator, expires } => {
                Cw721ExecuteMsg::ApproveAll { operator, expires }
            }
            ExecuteMsg::RevokeAll { operator } => Cw721ExecuteMsg::RevokeAll { operator },
            ExecuteMsg::Burn { token_id } => Cw721ExecuteMsg::Burn { token_id },
            _ => unreachable!("cannot convert {:?} to Cw721ExecuteMsg", msg),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },
    Approval {
        token_id: String,
        spender: String,
        include_expired: Option<bool>,
    },
    Approvals {
        token_id: String,
        include_expired: Option<bool>,
    },
    AllOperators {
        owner: String,
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    NumTokens {},
    ContractInfo {},
    NftInfo {
        token_id: String,
    },
    AllNftInfo {
        token_id: String,
        include_expired: Option<bool>,
    },
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    Minter {},
    // Custom query messages
    Locks {},
    TokenLocks {
        token_id: String,
    },
    MintedTokensPerAddress {
        address: String,
    },
    CollectionInfo {},
    Contracts {},
}

impl From<QueryMsg> for Cw721QueryMsg {
    fn from(msg: QueryMsg) -> Cw721QueryMsg {
        match msg {
            QueryMsg::OwnerOf {
                token_id,
                include_expired,
            } => Cw721QueryMsg::OwnerOf {
                token_id,
                include_expired,
            },
            QueryMsg::Approval {
                token_id,
                spender,
                include_expired,
            } => Cw721QueryMsg::Approval {
                token_id,
                spender,
                include_expired,
            },
            QueryMsg::Approvals {
                token_id,
                include_expired,
            } => Cw721QueryMsg::Approvals {
                token_id,
                include_expired,
            },
            QueryMsg::AllOperators {
                owner,
                include_expired,
                start_after,
                limit,
            } => Cw721QueryMsg::AllOperators {
                owner,
                include_expired,
                start_after,
                limit,
            },
            QueryMsg::NumTokens {} => Cw721QueryMsg::NumTokens {},
            QueryMsg::ContractInfo {} => Cw721QueryMsg::ContractInfo {},
            QueryMsg::NftInfo { token_id } => Cw721QueryMsg::NftInfo { token_id },
            QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            } => Cw721QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            },
            QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            } => Cw721QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            },
            QueryMsg::AllTokens { start_after, limit } => {
                Cw721QueryMsg::AllTokens { start_after, limit }
            }
            QueryMsg::Minter {} => Cw721QueryMsg::Minter {},
            _ => unreachable!("cannot convert {:?} to Cw721QueryMsg", msg),
        }
    }
}

impl From<TokenContractQueryMsg> for QueryMsg {
    fn from(msg: TokenContractQueryMsg) -> QueryMsg {
        match msg {
            TokenContractQueryMsg::OwnerOf {
                token_id,
                include_expired,
            } => QueryMsg::OwnerOf {
                token_id,
                include_expired,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LocksReponse {
    pub locks: Locks,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MintedTokenAmountResponse {
    pub amount: u32,
}
