use crate::state::{Config, FixedListing};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw20::Cw20ReceiveMsg;
use komple_framework_types::modules::marketplace::Listing;
use komple_framework_types::shared::execute::SharedExecuteMsg;
use komple_framework_types::shared::query::ResponseWrapper;

/// Message to be sent along the ```RegisterMsg``` for instantiation.
#[cw_serde]
pub struct InstantiateMsg {
    pub fund_info: MarketplaceFundInfo,
}

#[cw_serde]
pub struct MarketplaceFundInfo {
    pub is_native: bool,
    pub denom: String,
    pub cw20_address: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Admin message.
    ///
    /// Update the lock for buying.
    /// This applies for the normal buy operation.
    UpdateBuyLock {
        lock: bool,
    },
    /// Public message.
    ///
    /// List a new token for fixed amount sale.
    /// Every token created under parent hub
    /// can be listed for sale.
    ListFixedToken {
        collection_id: u32,
        token_id: u32,
        price: Uint128,
    },
    /// Public message.
    ///
    /// Remove a token from fixed amount sale.
    DelistFixedToken {
        collection_id: u32,
        token_id: u32,
    },
    /// Public message.
    ///
    /// Update the price of a listed token based on listing type.
    UpdatePrice {
        listing_type: Listing,
        collection_id: u32,
        token_id: u32,
        price: Uint128,
    },
    /// Public message.
    ///
    /// Buy a token that is listed on the marketplace.
    Buy {
        listing_type: Listing,
        collection_id: u32,
        token_id: u32,
    },
    /// Admin message.
    ///
    /// Same as ```Buy``` message but can be used with permissions.
    PermissionBuy {
        listing_type: Listing,
        collection_id: u32,
        token_id: u32,
        buyer: String,
    },
    /// Admin message.
    ///
    /// Update the operators of this contract.
    UpdateOperators {
        addrs: Vec<String>,
    },
    /// Hub message.
    ///
    /// Lock the execute entry point.
    /// Can only be called by the hub module.
    LockExecute {},
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
    Buy {
        listing_type: Listing,
        collection_id: u32,
        token_id: u32,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get the contract's config.
    #[returns(ResponseWrapper<Config>)]
    Config {},
    /// Get the fixed listing for a given collection and token id.
    #[returns(ResponseWrapper<FixedListing>)]
    FixedListing { collection_id: u32, token_id: u32 },
    /// Get the list of fixed token listings under a collection with pagination.
    #[returns(ResponseWrapper<Vec<FixedListing>>)]
    FixedListings {
        collection_id: u32,
        start_after: Option<u32>,
        limit: Option<u32>,
    },
    /// Get the operators of this contract.
    #[returns(ResponseWrapper<Vec<String>>)]
    Operators {},
}

#[cw_serde]
pub struct MigrateMsg {}
