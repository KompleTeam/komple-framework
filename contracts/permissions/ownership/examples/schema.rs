use cosmwasm_schema::write_api;
use komple_framework_ownership_permission::msg::{ExecuteMsg, QueryMsg};
use komple_types::shared::RegisterMsg;

fn main() {
    write_api! {
        instantiate: RegisterMsg,
        query: QueryMsg,
        execute: ExecuteMsg
    }
}
