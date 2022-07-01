use cosmwasm_std::{Empty, Binary};
use cw721::Expiration;
use cw721_base::{QueryMsg as Cw721QueryMsg, MintMsg, ExecuteMsg as Cw721ExecuteMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::Locks;

// pub type ExecuteMsg = cw721_base::ExecuteMsg<Empty>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg<T> {
        TransferNft { recipient: String, token_id: String },
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
        Revoke { spender: String, token_id: String },
        ApproveAll {
            operator: String,
            expires: Option<Expiration>,
        },
        RevokeAll { operator: String },
        Mint(MintMsg<T>),
        Burn { token_id: String },
        UpdateLocks {
            locks: Locks,
        },
}

impl From<ExecuteMsg<Empty>> for Cw721ExecuteMsg<Empty> {
    fn from(msg: ExecuteMsg<Empty>) -> Cw721ExecuteMsg<Empty> {
        match msg {
            ExecuteMsg::TransferNft {
                recipient,
                token_id
            } => Cw721ExecuteMsg::TransferNft { recipient, token_id },
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg
            } => Cw721ExecuteMsg::SendNft { contract, token_id, msg },
            ExecuteMsg::Approve {
                spender,
                token_id,
                expires
            } => Cw721ExecuteMsg::Approve { spender, token_id, expires },
            ExecuteMsg::Revoke {
                spender,
                token_id
            } => Cw721ExecuteMsg::Revoke { spender, token_id },
            ExecuteMsg::ApproveAll {
                operator,
                expires
            } => Cw721ExecuteMsg::ApproveAll { operator, expires },
            ExecuteMsg::RevokeAll { operator } => Cw721ExecuteMsg::RevokeAll { operator },
            ExecuteMsg::Mint(mint_msg) => Cw721ExecuteMsg::Mint(mint_msg.into()),
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
