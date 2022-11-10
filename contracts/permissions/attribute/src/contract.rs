#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use komple_metadata_module::helper::KompleMetadataModule;
use komple_types::modules::Modules;
use komple_types::modules::permission::AttributeConditions;
use komple_types::query::ResponseWrapper;
use komple_types::shared::{RegisterMsg, PARENT_ADDR_NAMESPACE};
use komple_utils::response::EventHelper;
use komple_utils::response::ResponseHelper;
use komple_utils::storage::StorageHelper;

use crate::error::ContractError;
use crate::msg::{AttributeMsg, AttributeTypes, ExecuteMsg, QueryMsg};
use crate::state::{Config, CONFIG, PERMISSION_MODULE_ADDR};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:komple-attribute-permission-module";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: RegisterMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = deps.api.addr_validate(&msg.admin)?;
    let config = Config {
        admin: admin.clone(),
    };
    CONFIG.save(deps.storage, &config)?;

    PERMISSION_MODULE_ADDR.save(deps.storage, &info.sender)?;

    Ok(
        ResponseHelper::new_permission("attribute", "instantiate").add_event(
            EventHelper::new("attribute_permission_instantiate")
                .add_attribute("admin", admin)
                .add_attribute("permission_module_addr", info.sender)
                .get(),
        ),
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Check { data } => execute_check(deps, env, info, data),
    }
}

pub fn execute_check(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    data: Binary,
) -> Result<Response, ContractError> {
    let permission_addr = PERMISSION_MODULE_ADDR.load(deps.storage)?;
    let hub_addr = StorageHelper::query_storage::<Addr>(
        &deps.querier,
        &permission_addr,
        PARENT_ADDR_NAMESPACE,
    )?;
    let mint_module_addr = StorageHelper::query_module_address(
        &deps.querier,
        &hub_addr.unwrap(),
        Modules::Mint.to_string(),
    )?;

    let msgs: Vec<AttributeMsg> = from_binary(&data)?;

    // TODO: Cache if the metadata is same
    for msg in msgs {
        // Get collection address
        let collection_addr = StorageHelper::query_collection_address(
            &deps.querier,
            &mint_module_addr,
            &msg.collection_id,
        )?;

        // Check metadata sub module in collection
        let sub_modules = StorageHelper::query_token_sub_modules(&deps.querier, &collection_addr)?;
        if sub_modules.metadata.is_none() {
            return Err(ContractError::MetadataNotFound {});
        };

        // Query token metadata to get attributes
        let response = KompleMetadataModule(sub_modules.metadata.unwrap())
            .query_metadata(&deps.querier, msg.token_id)?;
        let attributes = response.metadata.attributes;

        // Get the attribute value
        let attribute = attributes
            .into_iter()
            .find(|attr| attr.trait_type == msg.trait_type);

        // Check if attribute exists
        if msg.condition != AttributeConditions::Absent && attribute.is_none() {
            return Err(ContractError::AttributeNotFound {});
        }
        // If it is a comparison
        // Check if value is integer and both are same type
        if msg.condition == AttributeConditions::GreaterThan
            || msg.condition == AttributeConditions::GreaterThanOrEqual
            || msg.condition == AttributeConditions::LessThan
            || msg.condition == AttributeConditions::LessThanOrEqual
        {
            // If the types are not number, return error
            if get_value_type(&msg.value) != AttributeTypes::Integer
                && get_value_type(&attribute.as_ref().unwrap().value) != AttributeTypes::Integer
            {
                return Err(ContractError::AttributeTypeMismatch {});
            }
        }
        // Rest of the conditions
        match msg.condition {
            AttributeConditions::Absent => {
                if attribute.is_some() {
                    return Err(ContractError::AttributeFound {});
                }
            }
            AttributeConditions::Equal => {
                if attribute.unwrap().value != msg.value {
                    return Err(ContractError::AttributeNotEqual {});
                }
            }
            AttributeConditions::NotEqual => {
                if attribute.unwrap().value == msg.value {
                    return Err(ContractError::AttributeEqual {});
                }
            }
            AttributeConditions::GreaterThan => {
                let attribute_value = attribute.as_ref().unwrap().value.parse::<u32>()?;
                let msg_value = msg.value.parse::<u32>().unwrap();
                if attribute_value <= msg_value {
                    return Err(ContractError::AttributeLessThanOrEqual {});
                }
            }
            AttributeConditions::GreaterThanOrEqual => {
                let attribute_value = attribute.as_ref().unwrap().value.parse::<u32>()?;
                let msg_value = msg.value.parse::<u32>().unwrap();
                if attribute_value < msg_value {
                    return Err(ContractError::AttributeLessThan {});
                }
            }
            AttributeConditions::LessThan => {
                let attribute_value = attribute.as_ref().unwrap().value.parse::<u32>()?;
                let msg_value = msg.value.parse::<u32>().unwrap();
                if attribute_value >= msg_value {
                    return Err(ContractError::AttributeGreaterThanOrEqual {});
                }
            }
            AttributeConditions::LessThanOrEqual => {
                let attribute_value = attribute.as_ref().unwrap().value.parse::<u32>()?;
                let msg_value = msg.value.parse::<u32>().unwrap();
                if attribute_value > msg_value {
                    return Err(ContractError::AttributeGreaterThan {});
                }
            }
            _ => {}
        };
    }

    Ok(
        ResponseHelper::new_permission("attribute", "check").add_event(
            EventHelper::new("attribute_permission_check")
                // .add_attributes(event_attributes)
                .get(),
        ),
    )
}

// For now attribute comparison only works with
// Strings, integers and booleans
fn get_value_type(value: &str) -> AttributeTypes {
    if value.parse::<u32>().is_ok() {
        return AttributeTypes::Integer;
    }
    if value.parse::<bool>().is_ok() {
        return AttributeTypes::Boolean;
    }
    AttributeTypes::String
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ResponseWrapper<Config>> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ResponseWrapper {
        query: "config".to_string(),
        data: config,
    })
}
