use crate::msg::ExecuteMsg;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Addr, Coin, Empty, StdResult, WasmMsg};
use cw721_base::msg::ExecuteMsg as Cw721ExecuteMsg;
use komple_types::token::Locks;

#[cw_serde]
pub struct KompleTokenModule(pub Addr);

impl KompleTokenModule {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn update_collection_locks_msg(&self, locks: Locks) -> StdResult<WasmMsg> {
        let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
            msg: ExecuteMsg::UpdateLocks { locks },
        };
        Ok(WasmMsg::Execute {
            contract_addr: self.0.to_string(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })
    }

    pub fn update_token_locks_msg(&self, token_id: String, locks: Locks) -> StdResult<WasmMsg> {
        let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
            msg: ExecuteMsg::UpdateTokenLocks { token_id, locks },
        };
        Ok(WasmMsg::Execute {
            contract_addr: self.0.to_string(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })
    }

    pub fn admin_transfer_nft_msg(
        &self,
        token_id: String,
        recipient: String,
    ) -> StdResult<WasmMsg> {
        let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
            msg: ExecuteMsg::AdminTransferNft {
                recipient,
                token_id,
            },
        };
        Ok(WasmMsg::Execute {
            contract_addr: self.0.to_string(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })
    }

    pub fn burn_msg(&self, token_id: String) -> StdResult<WasmMsg> {
        let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
            msg: ExecuteMsg::Burn { token_id },
        };
        Ok(WasmMsg::Execute {
            contract_addr: self.0.to_string(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })
    }

    pub fn mint_msg(
        &self,
        owner: String,
        metadata_id: Option<u32>,
        funds: Vec<Coin>,
    ) -> StdResult<WasmMsg> {
        let msg: Cw721ExecuteMsg<Empty, ExecuteMsg> = Cw721ExecuteMsg::Extension {
            msg: ExecuteMsg::Mint { owner, metadata_id },
        };
        Ok(WasmMsg::Execute {
            contract_addr: self.0.to_string(),
            msg: to_binary(&msg)?,
            funds,
        })
    }
}
