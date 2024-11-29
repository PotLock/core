use crate::*;


pub fn calculate_required_storage_deposit(initial_storage_usage: u64) -> NearToken {
    let storage_used = env::storage_usage() - initial_storage_usage;
    log!("Storage used: {} bytes", storage_used);
    let required_cost = env::storage_byte_cost().checked_mul(storage_used as u128).unwrap();
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
        refund = refund.checked_sub(required_deposit).unwrap();
    } else {
        // storage was freed up; caller should be refunded for what they freed up, in addition to the deposit they sent
        let storage_freed = initial_storage_usage - env::storage_usage();
        let cost_freed = env::storage_byte_cost().checked_mul(storage_freed as u128).unwrap();
        refund = refund.checked_add(cost_freed).unwrap();
    }
    if refund > NearToken::from_yoctonear(1) {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}