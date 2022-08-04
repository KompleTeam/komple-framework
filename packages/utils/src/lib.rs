use cosmwasm_std::Addr;

pub fn have_admin_privilages(
    sender: &Addr,
    admin: &Addr,
    parent_contract: Option<&Addr>,
    enabled_modules: Option<&Vec<Addr>>,
) -> bool {
    if admin != sender {
        return false;
    }
    if parent_contract.is_some() && parent_contract.unwrap() != sender {
        return false;
    }
    if enabled_modules.is_some() && !enabled_modules.unwrap().contains(&sender) {
        return false;
    }
    true
}
