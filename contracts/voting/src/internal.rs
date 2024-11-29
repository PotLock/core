use near_sdk::NearToken;

use crate::*;

impl Contract {

    pub(crate) fn assert_at_least_one_yocto(&self) {
        assert!(
            env::attached_deposit() >= NearToken::from_near(1),
            "At least one yoctoNEAR must be attached"
        );
    }
    pub(crate) fn is_owner(&self, account_id: Option<&AccountId>) -> bool {
        account_id.unwrap_or(&env::predecessor_account_id()) == &self.owner
    }

    pub(crate) fn is_admin(&self, account_id: Option<&AccountId>) -> bool {
        self.admins
            .contains(account_id.unwrap_or(&env::predecessor_account_id()))
    }

    pub(crate) fn is_owner_or_admin(&self, account_id: Option<&AccountId>) -> bool {
        self.is_owner(account_id) || self.is_admin(account_id)
    }

    pub(crate) fn assert_admin_or_owner(&self) {
        assert!(
            self.is_owner_or_admin(None),
            "Only contract admin or owner can call this method"
        );
        // require caller to attach at least one yoctoNEAR for security purposes
        self.assert_at_least_one_yocto();
    }

    pub(crate) fn assert_not_paused(&self) {
        assert!(!self.paused, "Contract is paused");
    }

}
