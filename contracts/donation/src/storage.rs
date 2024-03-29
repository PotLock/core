use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn storage_deposit(&mut self) -> U128 {
        let mut deposit = env::attached_deposit();
        let initial_storage_usage = env::storage_usage();
        let existing_mapping = self.storage_deposits.get(&env::predecessor_account_id());
        if existing_mapping.is_none() {
            // insert record here and check how much storage was used, then subtract that cost from the deposit
            self.storage_deposits
                .insert(&env::predecessor_account_id(), &0);
            let storage_usage = env::storage_usage() - initial_storage_usage;
            let required_deposit = storage_usage as u128 * env::storage_byte_cost();
            assert!(
                deposit >= required_deposit,
                "The deposit is less than the required storage amount."
            );
            deposit -= required_deposit;
        }
        let account_id = env::predecessor_account_id();
        let storage_balance = self.storage_balance_of(&account_id);
        let new_storage_balance = storage_balance.0 + deposit;
        self.storage_deposits
            .insert(&account_id, &new_storage_balance);
        new_storage_balance.into()
    }

    pub fn storage_withdraw(&mut self, amount: Option<U128>) -> U128 {
        let account_id = env::predecessor_account_id();
        let storage_balance = self.storage_balance_of(&account_id);
        let amount = amount.map(|a| a.0).unwrap_or(storage_balance.0);
        assert!(
            amount <= storage_balance.0,
            "The withdrawal amount can't exceed the account storage balance."
        );
        let remainder = storage_balance.0 - amount;
        if remainder > 0 {
            self.storage_deposits.insert(&account_id, &remainder);
            Promise::new(account_id).transfer(amount);
        } else {
            // remove mapping and refund user for freed storage
            let initial_storage_usage = env::storage_usage();
            self.storage_deposits.remove(&account_id);
            let storage_usage = initial_storage_usage - env::storage_usage();
            let refund = storage_usage as u128 * env::storage_byte_cost();
            Promise::new(account_id).transfer(refund);
        }
        remainder.into()
    }

    pub fn storage_balance_of(&self, account_id: &AccountId) -> U128 {
        self.storage_deposits.get(account_id).unwrap_or(0).into()
    }
}
