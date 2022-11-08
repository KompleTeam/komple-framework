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

/// Event attributes for the metadata module.
pub enum MetadataEventAttributes {
    MetaInfo,
    Attributes,
}
impl MetadataEventAttributes {
    pub fn new_meta_info_value(field: &str, value: &Option<String>) -> String {
        format!("{}:{}", field, value.as_ref().unwrap_or(&String::from("")))
    }
    pub fn new_attribute_attribute(trait_type: String, value: String) -> Attribute {
        let value = format!("{}:{}", trait_type, value);
        Attribute::new("attributes", value)
    }
}
