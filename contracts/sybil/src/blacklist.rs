use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn blacklist_accounts(&mut self, accounts: Vec<AccountId>, reason: Option<String>) {
        self.assert_owner_or_admin();
        let initial_storage_usage = env::storage_usage();
        for account_id in accounts.clone() {
            self.blacklisted_accounts.insert(&account_id);
        }
        refund_deposit(initial_storage_usage);
        log_blacklist_accounts_event(&accounts, &reason);
    }

    #[payable]
    pub fn unblacklist_accounts(&mut self, accounts: Vec<AccountId>) {
        let initial_storage_usage = env::storage_usage();
        self.assert_owner_or_admin();
        for account_id in accounts.clone() {
            self.blacklisted_accounts.remove(&account_id);
        }
        refund_deposit(initial_storage_usage);
        log_unblacklist_accounts_event(&accounts);
    }

    pub fn get_blacklisted_accounts(&self) -> Vec<AccountId> {
        self.blacklisted_accounts.iter().collect()
    }
}
