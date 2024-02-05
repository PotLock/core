use crate::*;

pub(crate) fn account_vec_to_set(
    account_vec: Vec<AccountId>,
    storage_key: StorageKey,
) -> UnorderedSet<AccountId> {
    let mut set = UnorderedSet::new(storage_key);
    for element in account_vec.iter() {
        set.insert(element);
    }
    set
}

pub fn calculate_required_storage_deposit(initial_storage_usage: u64) -> Balance {
    let storage_used = env::storage_usage() - initial_storage_usage;
    log!("Storage used: {} bytes", storage_used);
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    required_cost
}

pub fn refund_deposit(initial_storage_usage: u64) {
    let attached_deposit = env::attached_deposit();
    let mut refund = attached_deposit;
    if env::storage_usage() > initial_storage_usage {
        // caller should pay for the extra storage they used and be refunded for the rest
        // let storage_used = env::storage_usage() - initial_storage_usage;
        let required_deposit = calculate_required_storage_deposit(initial_storage_usage);
        // env::storage_byte_cost() * Balance::from(storage_used);
        require!(
            required_deposit <= attached_deposit,
            format!(
                "Must attach {} yoctoNEAR to cover storage",
                required_deposit
            )
        );
        refund -= required_deposit;
    } else {
        // storage was freed up; caller should be refunded for what they freed up, in addition to the deposit they sent
        let storage_freed = initial_storage_usage - env::storage_usage();
        let cost_freed = env::storage_byte_cost() * Balance::from(storage_freed);
        refund += cost_freed;
    }
    if refund > 0 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}
