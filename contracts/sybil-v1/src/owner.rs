use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn owner_change_owner(&mut self, new_owner: AccountId) {
        self.assert_owner();
        self.owner = new_owner.into();
    }

    #[payable]
    pub fn owner_add_admins(&mut self, account_ids: Vec<AccountId>) {
        self.assert_owner();
        for account_id in account_ids {
            self.admins.insert(&account_id);
        }
    }

    #[payable]
    pub fn owner_remove_admins(&mut self, account_ids: Vec<AccountId>) {
        self.assert_owner();
        for account_id in account_ids {
            self.admins.remove(&account_id);
        }
    }
}
