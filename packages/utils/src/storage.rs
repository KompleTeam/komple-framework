use cosmwasm_std::{from_slice, Addr, Decimal, Empty, QuerierWrapper, StdError, StdResult};
use cw721_base::state::TokenInfo;
use cw_storage_plus::Path;
use komple_types::{
    collection::{COLLECTION_ADDRS_NAMESPACE, LINKED_COLLECTIONS_NAMESPACE},
    fee::{FixedPayment, FIXED_FEES_NAMESPACE, PERCENTAGE_FEES_NAMESPACE},
    module::MODULES_NAMESPACE,
    token::{
        Locks, SubModules, LOCKS_NAMESPACE, SUB_MODULES_NAMESPACE, TOKENS_NAMESPACE,
        TOKEN_LOCKS_NAMESPACE,
    },
};
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{ops::Deref, str::from_utf8};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PercentagePayment {
    pub address: Option<String>,
    pub value: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct StorageHelper();

impl StorageHelper {
    // namespace -> storage key
    // key_name -> item key
    pub fn get_map_storage_key(namepspace: &str, key_bytes: &[&[u8]]) -> StdResult<String> {
        let namespace_bytes = namepspace.as_bytes();
        let path: Path<Vec<u32>> = Path::new(namespace_bytes, key_bytes);
        let path_str = from_utf8(path.deref())?;
        Ok(path_str.to_string())
    }

    // To find the key value in storage, we need to construct a path to the key
    // For Map storage this key is generated with get_map_storage_key
    // For Item storage this key is the namespace value
    pub fn query_storage<T>(
        querier: &QuerierWrapper,
        addr: &Addr,
        key: &str,
    ) -> StdResult<Option<T>>
    where
        T: DeserializeOwned,
    {
        let data = querier.query_wasm_raw(addr, key.as_bytes())?;
        match data {
            Some(data) => {
                let res = from_utf8(&data)?;
                let res = from_slice(res.as_bytes())?;
                Ok(Some(res))
            }
            None => Ok(None),
        }
    }

    pub fn query_module_address(
        querier: &QuerierWrapper,
        hub_addr: &Addr,
        module: String,
    ) -> StdResult<Addr> {
        let key = Self::get_map_storage_key(MODULES_NAMESPACE, &[module.as_bytes()])?;
        let res = Self::query_storage::<Addr>(querier, hub_addr, &key)?;
        match res {
            Some(res) => Ok(res),
            None => Err(StdError::NotFound {
                kind: "Module".to_string(),
            }),
        }
    }

    pub fn query_collection_address(
        querier: &QuerierWrapper,
        mint_module_address: &Addr,
        collection_id: &u32,
    ) -> StdResult<Addr> {
        let key =
            Self::get_map_storage_key(COLLECTION_ADDRS_NAMESPACE, &[&collection_id.to_be_bytes()])?;
        let res = Self::query_storage::<Addr>(querier, mint_module_address, &key)?;
        match res {
            Some(res) => Ok(res),
            None => Err(StdError::NotFound {
                kind: "Collection".to_string(),
            }),
        }
    }

    pub fn query_linked_collections(
        querier: &QuerierWrapper,
        mint_module_address: &Addr,
        collection_id: u32,
    ) -> StdResult<Vec<u32>> {
        let key = Self::get_map_storage_key(
            LINKED_COLLECTIONS_NAMESPACE,
            &[&collection_id.to_be_bytes()],
        )?;
        let res = Self::query_storage::<Vec<u32>>(querier, mint_module_address, &key)?;
        match res {
            Some(res) => Ok(res),
            None => Ok(vec![]),
        }
    }

    pub fn query_token_owner(
        querier: &QuerierWrapper,
        collection_addr: &Addr,
        token_id: &u32,
    ) -> StdResult<Addr> {
        let key = Self::get_map_storage_key(TOKENS_NAMESPACE, &[token_id.to_string().as_bytes()])?;
        let res = Self::query_storage::<TokenInfo<Empty>>(querier, collection_addr, &key)?;
        match res {
            Some(res) => Ok(Addr::unchecked(res.owner)),
            None => Err(StdError::NotFound {
                kind: "Token".to_string(),
            }),
        }
    }

    pub fn query_collection_locks(
        querier: &QuerierWrapper,
        collection_addr: &Addr,
    ) -> StdResult<Locks> {
        let res = Self::query_storage::<Locks>(querier, collection_addr, LOCKS_NAMESPACE)?;
        match res {
            Some(res) => Ok(res),
            None => Err(StdError::NotFound {
                kind: "Locks".to_string(),
            }),
        }
    }

    pub fn query_token_locks(
        querier: &QuerierWrapper,
        collection_addr: &Addr,
        token_id: &u32,
    ) -> StdResult<Locks> {
        let key =
            Self::get_map_storage_key(TOKEN_LOCKS_NAMESPACE, &[token_id.to_string().as_bytes()])?;
        let res = Self::query_storage::<Locks>(querier, collection_addr, &key)?;
        match res {
            Some(res) => Ok(res),
            None => Ok(Locks {
                mint_lock: false,
                burn_lock: false,
                transfer_lock: false,
                send_lock: false,
            }),
        }
    }

    pub fn query_fixed_fee(
        querier: &QuerierWrapper,
        fee_module_addr: &Addr,
        module_name: String,
        fee_name: String,
    ) -> StdResult<FixedPayment> {
        let key = Self::get_map_storage_key(
            FIXED_FEES_NAMESPACE,
            &[module_name.as_bytes(), fee_name.as_bytes()],
        )?;
        let res = Self::query_storage::<FixedPayment>(querier, fee_module_addr, &key)?;
        match res {
            Some(res) => Ok(res),
            None => Err(StdError::NotFound {
                kind: "Fixed payment".to_string(),
            }),
        }
    }

    pub fn query_percentage_fee(
        querier: &QuerierWrapper,
        fee_module_addr: &Addr,
        module_name: String,
        fee_name: String,
    ) -> StdResult<PercentagePayment> {
        let key = Self::get_map_storage_key(
            PERCENTAGE_FEES_NAMESPACE,
            &[module_name.as_bytes(), fee_name.as_bytes()],
        )?;
        let res = Self::query_storage::<PercentagePayment>(querier, fee_module_addr, &key)?;
        match res {
            Some(res) => Ok(res),
            None => Err(StdError::NotFound {
                kind: "Percentage payment".to_string(),
            }),
        }
    }

    pub fn query_token_sub_modules(
        querier: &QuerierWrapper,
        token_module_addr: &Addr,
    ) -> StdResult<SubModules> {
        let res =
            Self::query_storage::<SubModules>(querier, token_module_addr, SUB_MODULES_NAMESPACE)?;
        match res {
            Some(res) => Ok(res),
            None => Err(StdError::NotFound {
                kind: "Sub modules".to_string(),
            }),
        }
    }
}
