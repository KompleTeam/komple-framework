use crate::{
    msg::{ExecuteMsg, MetadataResponse, QueryMsg},
    state::{MetaInfo, Trait},
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Addr, QuerierWrapper, StdResult, WasmMsg};
use komple_types::query::ResponseWrapper;

/// Helper methods for the metadata module.
///
/// Used for constructing some of the execute messages and performing queries.
#[cw_serde]
pub struct KompleMetadataModule(pub Addr);

impl KompleMetadataModule {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn add_metadata_msg(
        &self,
        meta_info: MetaInfo,
        attributes: Vec<Trait>,
    ) -> StdResult<WasmMsg> {
        let msg = ExecuteMsg::AddMetadata {
            meta_info,
            attributes,
        };
        Ok(WasmMsg::Execute {
            contract_addr: self.0.to_string(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })
    }

    pub fn link_metadata_msg(&self, token_id: u32, metadata_id: Option<u32>) -> StdResult<WasmMsg> {
        let msg = ExecuteMsg::LinkMetadata {
            token_id,
            metadata_id,
        };
        Ok(WasmMsg::Execute {
            contract_addr: self.0.to_string(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })
    }

    pub fn unlink_metadata_msg(&self, token_id: u32) -> StdResult<WasmMsg> {
        let msg = ExecuteMsg::UnlinkMetadata { token_id };
        Ok(WasmMsg::Execute {
            contract_addr: self.0.to_string(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })
    }

    // Queries
    pub fn query_metadata(
        &self,
        querier: &QuerierWrapper,
        token_id: u32,
    ) -> StdResult<MetadataResponse> {
        let msg = QueryMsg::Metadata { token_id };
        let res: ResponseWrapper<MetadataResponse> =
            querier.query_wasm_smart(self.0.to_string(), &msg)?;
        Ok(res.data)
    }
}
