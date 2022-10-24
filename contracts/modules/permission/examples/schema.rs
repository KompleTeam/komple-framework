use cosmwasm_schema::write_api;
use komple_permission_module::msg::{ExecuteMsg, MigrateMsg, QueryMsg};
use komple_types::hub::RegisterMsg;

fn main() {
    write_api! {
        instantiate: RegisterMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg
    }
}
