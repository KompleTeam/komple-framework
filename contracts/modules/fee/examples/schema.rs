use cosmwasm_schema::write_api;
use komple_fee_module::msg::{ExecuteMsg, QueryMsg};
use komple_types::hub::RegisterMsg;

fn main() {
    write_api! {
        instantiate: RegisterMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
    }
}
