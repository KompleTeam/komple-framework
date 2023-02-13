use crate::state::HubInfo;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;
use komple_framework_types::shared::query::ResponseWrapper;

/// Message to be sent along the ```RegisterMsg``` for instantiation.
#[cw_serde]
pub struct InstantiateMsg {
    /// Hub info is the general information about the hub.
    pub hub_info: HubInfo,
    /// Marbu fee module is the optional address for defining the fee module address for Marbu projects.
    pub marbu_fee_module: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Admin message.
    ///
    /// Adds a new module to the hub registry.
    /// Saves the module address to it's storage.
    RegisterModule {
        code_id: u64,
        module: String,
        msg: Option<Binary>,
    },
    /// Admin message.
    ///
    /// Updates the general hub information.
    UpdateHubInfo {
        name: String,
        description: String,
        image: String,
        external_link: Option<String>,
    },
    /// Admin message.
    ///
    /// Removes a module from the hub module registry.
    DeregisterModule { module: String },
    /// Admin message.
    ///
    /// Updates the operators of this contract.
    UpdateOperators { addrs: Vec<String> },
    /// Admin message.
    ///
    /// Migrate the contracts available in framework.
    MigrateContracts {
        code_id: u64,
        contract_address: String,
        msg: Binary,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Gets the contract's config.
    #[returns(ResponseWrapper<ConfigResponse>)]
    Config {},
    /// Resolves the module address for the given module.
    #[returns(ResponseWrapper<String>)]
    ModuleAddress { module: String },
    /// Lists the module names and addresses registered in the hub.
    #[returns(ResponseWrapper<Vec<ModulesResponse>>)]
    Modules {
        start_after: Option<String>,
        limit: Option<u8>,
    },
    /// Gets the operators of this contract.
    #[returns(ResponseWrapper<Vec<String>>)]
    Operators {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub hub_info: HubInfo,
}

#[cw_serde]
pub struct ModulesResponse {
    pub name: String,
    pub address: String,
}

#[cw_serde]
pub struct MigrateMsg {}
