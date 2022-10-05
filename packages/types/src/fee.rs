use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum Fees {
    Fixed,
    Percentage,
}

impl Fees {
    pub fn as_str(&self) -> &'static str {
        match self {
            Fees::Fixed => "fixed",
            Fees::Percentage => "percentage",
        }
    }
}

pub const FIXED_FEES_NAMESPACE: &str = "fixed_fees";

pub const PERCENTAGE_FEES_NAMESPACE: &str = "percentage_fees";
