use cosmwasm_schema::write_api;
use komple_framework_marketplace_module::msg::{ExecuteMsg, MigrateMsg, QueryMsg};
use komple_framework_types::shared::RegisterMsg;

fn main() {
    write_api! {
        instantiate: RegisterMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg
    }
}
