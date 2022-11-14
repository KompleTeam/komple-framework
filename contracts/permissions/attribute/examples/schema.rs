use cosmwasm_schema::write_api;
use komple_framework_attribute_permission::msg::{ExecuteMsg, QueryMsg};
use komple_framework_types::shared::RegisterMsg;

fn main() {
    write_api! {
        instantiate: RegisterMsg,
        query: QueryMsg,
        execute: ExecuteMsg
    }
}
