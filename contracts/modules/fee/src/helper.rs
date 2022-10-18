use crate::msg::{CustomPaymentAddress, ExecuteMsg, QueryMsg};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Addr, Coin, Decimal, QuerierWrapper, StdResult, Uint128, WasmMsg};
use komple_types::fee::Fees;
use komple_types::query::ResponseWrapper;

#[cw_serde]
pub struct KompleFeeModule(pub Addr);

impl KompleFeeModule {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn distribute_msg(
        &self,
        fee_type: Fees,
        module_name: String,
        custom_payment_addresses: Option<Vec<CustomPaymentAddress>>,
        funds: Vec<Coin>,
    ) -> StdResult<WasmMsg> {
        let msg = ExecuteMsg::Distribute {
            fee_type,
            module_name,
            custom_payment_addresses,
        };
        Ok(WasmMsg::Execute {
            contract_addr: self.0.to_string(),
            msg: to_binary(&msg)?,
            funds,
        })
    }

    // Queries
    pub fn query_total_percentage_fees(
        &self,
        querier: &QuerierWrapper,
        module_name: &str,
    ) -> StdResult<Decimal> {
        let msg = QueryMsg::TotalPercentageFees {
            module_name: module_name.to_string(),
        };
        let res: ResponseWrapper<Decimal> =
            querier.query_wasm_smart(self.addr().to_string(), &msg)?;
        Ok(res.data)
    }

    pub fn query_total_fixed_fees(
        &self,
        querier: &QuerierWrapper,
        module_name: &str,
    ) -> StdResult<Uint128> {
        let msg = QueryMsg::TotalFixedFees {
            module_name: module_name.to_string(),
        };
        let res: ResponseWrapper<Uint128> =
            querier.query_wasm_smart(self.addr().to_string(), &msg)?;
        Ok(res.data)
    }
}
