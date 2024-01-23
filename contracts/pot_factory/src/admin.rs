use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn owner_change_owner(&mut self, new_owner: AccountId) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        self.owner = new_owner;
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn owner_set_admins(&mut self, account_ids: Vec<AccountId>) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        self.admins.clear();
        for account_id in account_ids {
            self.admins.insert(&account_id);
        }
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn owner_add_admins(&mut self, account_ids: Vec<AccountId>) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        for account_id in account_ids {
            self.admins.insert(&account_id);
        }
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn owner_remove_admins(&mut self, account_ids: Vec<AccountId>) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        for account_id in account_ids {
            self.admins.remove(&account_id);
        }
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn owner_clear_admins(&mut self) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        self.admins.clear();
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_protocol_fee_basis_points(&mut self, protocol_fee_basis_points: u32) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.protocol_fee_basis_points = protocol_fee_basis_points;
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_protocol_fee_recipient_account(
        &mut self,
        protocol_fee_recipient_account: AccountId,
    ) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.protocol_fee_recipient_account = protocol_fee_recipient_account;
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_protocol_config(
        &mut self,
        protocol_fee_basis_points: u32,
        protocol_fee_recipient_account: AccountId,
    ) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.protocol_fee_basis_points = protocol_fee_basis_points;
        self.protocol_fee_recipient_account = protocol_fee_recipient_account;
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_default_chef_fee_basis_points(&mut self, default_chef_fee_basis_points: u32) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.default_chef_fee_basis_points = default_chef_fee_basis_points;
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_add_whitelisted_deployers(&mut self, whitelisted_deployers: Vec<AccountId>) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        for account_id in whitelisted_deployers {
            self.whitelisted_deployers.insert(&account_id);
        }
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_remove_whitelisted_deployers(&mut self, whitelisted_deployers: Vec<AccountId>) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        for account_id in whitelisted_deployers {
            self.whitelisted_deployers.remove(&account_id);
        }
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_require_whitelist(&mut self, require_whitelist: bool) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.require_whitelist = require_whitelist;
        refund_deposit(initial_storage_usage);
    }
}
