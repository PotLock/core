use crate::*;

pub(crate) fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("-")
}

pub fn calculate_required_storage_deposit(initial_storage_usage: u64) -> Balance {
    let storage_used = env::storage_usage() - initial_storage_usage;
    log!("Storage used: {} bytes", storage_used);
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    required_cost
}
