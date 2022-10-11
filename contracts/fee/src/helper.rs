use crate::msg::{CustomPaymentAddress, ExecuteMsg};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Addr, Coin, StdResult, WasmMsg};
use komple_types::fee::Fees;

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
}
