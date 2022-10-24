use cosmwasm_schema::write_api;
use komple_ownership_permission_module::msg::{ExecuteMsg, QueryMsg};
use komple_types::hub::RegisterMsg;

fn main() {
    write_api! {
        instantiate: RegisterMsg,
        query: QueryMsg,
        execute: ExecuteMsg
    }
}
