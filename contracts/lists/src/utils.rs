use crate::*;

pub fn refund_deposit(initial_storage_usage: u64) {
    let attached_deposit = env::attached_deposit();
    let mut refund = attached_deposit;
    if env::storage_usage() > initial_storage_usage {
        // caller should pay for the extra storage they used and be refunded for the rest
        let storage_used = env::storage_usage() - initial_storage_usage;
        log!("Storage used: {} bytes", storage_used);
        let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
        require!(
            required_cost <= attached_deposit,
            format!("Must attach {} yoctoNEAR to cover storage", required_cost)
        );
        refund -= required_cost;
    } else {
        // storage was freed up; caller should be refunded for what they freed up, in addition to the deposit they sent
        let storage_freed = initial_storage_usage - env::storage_usage();
        log!("Storage freed: {} bytes", storage_freed);
        let cost_freed = env::storage_byte_cost() * Balance::from(storage_freed);
        refund += cost_freed;
    }
    if refund > 1 {
        log!("Refunding {} yoctoNEAR", refund);
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}
