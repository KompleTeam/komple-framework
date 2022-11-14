use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, QuerierWrapper, StdResult};
use komple_framework_types::shared::query::ResponseWrapper;

use crate::msg::{ConfigResponse, QueryMsg};

#[cw_serde]
pub struct KompleWhitelistHelper {
    addr: Addr,
}

impl KompleWhitelistHelper {
    pub fn new(addr: Addr) -> KompleWhitelistHelper {
        Self { addr }
    }

    // Queries
    pub fn query_is_active(&self, querier: &QuerierWrapper) -> StdResult<bool> {
        let msg = QueryMsg::IsActive {};
        let res: ResponseWrapper<bool> = querier.query_wasm_smart(self.addr.to_string(), &msg)?;
        Ok(res.data)
    }

    pub fn query_config(&self, querier: &QuerierWrapper) -> StdResult<ConfigResponse> {
        let msg = QueryMsg::Config {};
        let res: ResponseWrapper<ConfigResponse> =
            querier.query_wasm_smart(self.addr.to_string(), &msg)?;
        Ok(res.data)
    }
}
