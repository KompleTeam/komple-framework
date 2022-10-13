use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct ResponseWrapper<T> {
    pub query: String,
    pub data: T,
}
impl<T> ResponseWrapper<T> {
    pub fn new(query: &str, data: T) -> Self {
        Self {
            query: query.to_string(),
            data,
        }
    }
}
