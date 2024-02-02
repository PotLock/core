use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn owner_change_owner(&mut self, owner: AccountId) {
        self.assert_owner();
        self.owner = owner;
    }
}
