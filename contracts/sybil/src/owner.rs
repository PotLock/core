use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn owner_change_owner(&mut self, new_owner: AccountId) {
        let initial_storage_usage = env::storage_usage();
        self.assert_owner();
        self.owner = new_owner.clone().into();
        refund_deposit(initial_storage_usage);
        log_transfer_owner_event(&new_owner);
    }

    #[payable]
    pub fn owner_add_admins(&mut self, account_ids: Vec<AccountId>) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        for account_id in account_ids {
            self.admins.insert(&account_id);
        }
        refund_deposit(initial_storage_usage);
        log_update_admins_event(&self.admins.iter().collect());
    }

    #[payable]
    pub fn owner_remove_admins(&mut self, account_ids: Vec<AccountId>) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        for account_id in account_ids {
            self.admins.remove(&account_id);
        }
        refund_deposit(initial_storage_usage);
        log_update_admins_event(&self.admins.iter().collect());
    }
}
