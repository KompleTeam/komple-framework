use crate::msg::{ExecuteMsg, QueryMsg};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Addr, Coin, StdResult, WasmMsg, QuerierWrapper};

#[cw_serde]
pub struct KompleMintModule(pub Addr);

impl KompleMintModule {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn mint_to_msg(
        &self,
        recipient: String,
        collection_id: u32,
        metadata_id: Option<u32>,
        funds: Vec<Coin>,
    ) -> StdResult<WasmMsg> {
        let msg = ExecuteMsg::AdminMint {
            recipient,
            collection_id,
            metadata_id,
        };
        Ok(WasmMsg::Execute {
            contract_addr: self.0.to_string(),
            msg: to_binary(&msg)?,
            funds,
        })
    }
}
