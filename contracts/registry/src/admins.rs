use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn owner_add_admins(&mut self, admins: Vec<AccountId>) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        for admin in admins {
            self.admins.insert(&admin);
        }
        refund_deposit(initial_storage_usage);
    }

    pub fn get_admins(&self) -> Vec<AccountId> {
        self.admins.to_vec()
    }

    #[payable]
    pub fn owner_remove_admins(&mut self, admins: Vec<AccountId>) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        for admin in admins {
            self.admins.remove(&admin);
        }
        refund_deposit(initial_storage_usage);
    }
}
