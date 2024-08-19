use crate::*;

#[near]
impl Contract {
    // OWNER
    #[payable]
    pub fn owner_change_owner(&mut self, owner: AccountId) {
        // TODO: consider renaming to owner_set_owner, but currently deployed Registry uses owner_change_owner.
        self.assert_contract_owner();
        let initial_storage_usage = env::storage_usage();
        self.owner = owner;
        refund_deposit(initial_storage_usage);
        log_config_update_event(&self.get_config());
    }

    pub fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    #[payable]
    pub fn owner_add_admins(&mut self, admins: Vec<AccountId>) {
        self.assert_contract_owner();
        let initial_storage_usage = env::storage_usage();
        for account_id in admins {
            self.admins.insert(account_id);
        }
        refund_deposit(initial_storage_usage);
        log_config_update_event(&self.get_config());
    }

    #[payable]
    pub fn owner_remove_admins(&mut self, admins: Vec<AccountId>) {
        self.assert_contract_owner();
        let initial_storage_usage = env::storage_usage();
        for account_id in admins {
            self.admins.remove(&account_id);
        }
        refund_deposit(initial_storage_usage);
        log_config_update_event(&self.get_config());
    }

    #[payable]
    pub fn owner_clear_admins(&mut self) {
        self.assert_contract_owner();
        let initial_storage_usage = env::storage_usage();
        self.admins.clear();
        refund_deposit(initial_storage_usage);
        let config = self.get_config();
        log_config_update_event(&config);
    }
}
