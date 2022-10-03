use crate::state::{CollectionInfo, Contracts};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Decimal, Empty, Timestamp, Uint128};
use cw721::{
    AllNftInfoResponse, ApprovalResponse, ApprovalsResponse, ContractInfoResponse, Expiration,
    NftInfoResponse, NumTokensResponse, OperatorsResponse, OwnerOfResponse, TokensResponse,
};
use cw721_base::{ExecuteMsg as Cw721ExecuteMsg, MinterResponse, QueryMsg as Cw721QueryMsg};
use komple_types::{metadata::Metadata as MetadataType, query::ResponseWrapper, tokens::Locks};
use komple_whitelist_module::msg::InstantiateMsg as WhitelistInstantiateMsg;

#[cw_serde]
pub struct TokenInfo {
    pub symbol: String,
    pub minter: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub creator: String,
    pub token_info: TokenInfo,
    pub per_address_limit: Option<u32>,
    pub start_time: Option<Timestamp>,
    pub collection_info: CollectionInfo,
    pub max_token_limit: Option<u32>,
    pub unit_price: Option<Uint128>,
    pub native_denom: String,
    pub royalty_share: Option<Decimal>,
}

#[cw_serde]
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
        metadata_id: Option<u32>,
    },
    Burn {
        token_id: String,
    },

    // ADMIN MESSAGES
    UpdateOperators {
        addrs: Vec<String>,
    },
    UpdateRoyaltyShare {
        royalty_share: Option<Decimal>,
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

    // CONFIG MESSAGES
    UpdatePerAddressLimit {
        per_address_limit: Option<u32>,
    },
    UpdateStartTime {
        start_time: Option<Timestamp>,
    },

    // CONTRACT MESSAGES
    InitMetadataContract {
        code_id: u64,
        metadata_type: MetadataType,
    },
    InitWhitelistContract {
        code_id: u64,
        instantiate_msg: WhitelistInstantiateMsg,
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

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(OwnerOfResponse)]
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },
    #[returns(ApprovalResponse)]
    Approval {
        token_id: String,
        spender: String,
        include_expired: Option<bool>,
    },
    #[returns(ApprovalsResponse)]
    Approvals {
        token_id: String,
        include_expired: Option<bool>,
    },
    #[returns(OperatorsResponse)]
    AllOperators {
        owner: String,
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(NumTokensResponse)]
    NumTokens {},
    #[returns(ContractInfoResponse)]
    ContractInfo {},
    #[returns(NftInfoResponse<Empty>)]
    NftInfo { token_id: String },
    #[returns(AllNftInfoResponse<Empty>)]
    AllNftInfo {
        token_id: String,
        include_expired: Option<bool>,
    },
    #[returns(TokensResponse)]
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(TokensResponse)]
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(MinterResponse)]
    Minter {},
    // Custom query messages
    #[returns(ResponseWrapper<Locks>)]
    Locks {},
    #[returns(ResponseWrapper<Locks>)]
    TokenLocks { token_id: String },
    #[returns(ResponseWrapper<u32>)]
    MintedTokensPerAddress { address: String },
    #[returns(ResponseWrapper<CollectionInfo>)]
    CollectionInfo {},
    #[returns(ResponseWrapper<Contracts>)]
    Contracts {},
    #[returns(ResponseWrapper<ConfigResponse>)]
    Config {},
    #[returns(ResponseWrapper<Vec<String>>)]
    ContractOperators {},
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

#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub creator: String,
    pub native_denom: String,
    pub per_address_limit: Option<u32>,
    pub start_time: Option<Timestamp>,
    pub max_token_limit: Option<u32>,
    pub unit_price: Option<Uint128>,
    pub royalty_share: Option<Decimal>,
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
