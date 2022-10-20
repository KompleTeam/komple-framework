use crate::state::Config;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;
use komple_types::query::ResponseWrapper;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateMergeLock {
        lock: bool,
    },
    Merge {
        msg: Binary,
    },
    PermissionMerge {
        permission_msg: Binary,
        merge_msg: Binary,
    },
    UpdateOperators {
        addrs: Vec<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ResponseWrapper<Config>)]
    Config {},
    #[returns(ResponseWrapper<Vec<String>>)]
    Operators {},
}

#[cw_serde]
pub struct MergeBurnMsg {
    pub collection_id: u32,
    pub token_id: u32,
}

#[cw_serde]
pub struct MergeMsg {
    pub mint_id: u32,
    pub metadata_id: Option<u32>,
    pub burn_ids: Vec<MergeBurnMsg>,
}

#[cw_serde]
pub struct MigrateMsg {}
