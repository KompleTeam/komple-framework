use cosmwasm_std::{Addr, Attribute, DepsMut, MessageInfo, Response, StdError, StdResult};
use cw_storage_plus::Item;
use komple_types::shared::{HUB_ADDR_NAMESPACE, PARENT_ADDR_NAMESPACE};
use thiserror::Error;

use crate::{
    check_admin_privileges,
    response::{EventHelper, ResponseHelper},
    storage::StorageHelper,
    UtilError,
};

pub fn execute_lock_execute(
    deps: DepsMut,
    info: MessageInfo,
    module: &str,
    module_addr: &Addr,
    execute_lock_state: Item<bool>,
) -> Result<Response, SharedError> {
    if let Some(hub_addr) =
        StorageHelper::query_storage::<Addr>(&deps.querier, module_addr, HUB_ADDR_NAMESPACE)?
    {
        if hub_addr != info.sender {
            return Err(SharedError::Unauthorized {});
        };

        execute_lock_state.save(deps.storage, &true)?;

        Ok(ResponseHelper::new_module(module, "lock_execute"))
    } else {
        return Err(SharedError::Unauthorized {});
    }
}

pub fn execute_update_operators(
    deps: DepsMut,
    info: MessageInfo,
    module: &str,
    module_addr: &Addr,
    admin: &Addr,
    operators_state: Item<Vec<Addr>>,
    mut addrs: Vec<String>,
) -> Result<Response, SharedError> {
    let parent_addr =
        StorageHelper::query_storage::<Addr>(&deps.querier, module_addr, PARENT_ADDR_NAMESPACE)?;
    let operators = operators_state.may_load(deps.storage)?;

    check_admin_privileges(&info.sender, module_addr, &admin, parent_addr, operators)?;

    addrs.sort_unstable();
    addrs.dedup();

    let mut event_attributes: Vec<Attribute> = vec![];

    let addrs = addrs
        .iter()
        .map(|addr| -> StdResult<Addr> {
            let addr = deps.api.addr_validate(addr)?;
            event_attributes.push(Attribute {
                key: "addrs".to_string(),
                value: addr.to_string(),
            });
            Ok(addr)
        })
        .collect::<StdResult<Vec<Addr>>>()?;

    operators_state.save(deps.storage, &addrs)?;

    Ok(
        ResponseHelper::new_module(module, "update_operators").add_event(
            EventHelper::new(format!("{}_update_operators", module))
                .add_attributes(event_attributes)
                .get(),
        ),
    )
}

#[derive(Error, Debug, PartialEq)]
pub enum SharedError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("{0}")]
    UtilError(#[from] UtilError),
}
