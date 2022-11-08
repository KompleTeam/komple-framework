use cosmwasm_std::Attribute;

/// Event attributes for the merge module.
pub enum MergeEventAttributes {
    BurnIds,
}
impl MergeEventAttributes {
    pub fn new_burn_id_attribute(collection_id: u32, token_id: u32) -> Attribute {
        let value = format!("{}:{}", collection_id, token_id);
        Attribute::new("burn_ids", value)
    }
}
