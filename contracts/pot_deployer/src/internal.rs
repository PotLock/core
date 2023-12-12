use crate::*;

impl Contract {
    pub(crate) fn is_owner(&self) -> bool {
        env::predecessor_account_id() == self.owner
    }

    pub(crate) fn is_admin(&self) -> bool {
        self.admins.contains(&env::predecessor_account_id())
    }

    pub(crate) fn assert_owner(&self) {
        assert!(self.is_owner(), "Only contract owner can call this method");
    }

    pub(crate) fn assert_admin_or_greater(&self) {
        assert!(
            self.is_admin() || self.is_owner(),
            "Only contract admin or owner can call this method"
        );
    }

    pub(crate) fn is_whitelisted_deployer(&self) -> bool {
        self.whitelisted_deployers
            .contains(&env::predecessor_account_id())
    }

    pub(crate) fn assert_admin_or_whitelisted_deployer(&self) {
        assert!(
            self.is_owner() || self.is_admin() || self.is_whitelisted_deployer(),
            "Only contract admin or whitelisted deployer can call this method"
        );
    }
}
