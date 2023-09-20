use crate::*;

impl Contract {
    pub(crate) fn assert_admin(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.admin,
            "Only admin can call this method"
        );
    }

    pub(crate) fn assert_admin_or_whitelisted_deployer(&self) {
        assert!(
            env::predecessor_account_id() == self.admin
                || self
                    .whitelisted_deployers
                    .contains(&env::predecessor_account_id()),
            "Only admin or whitelisted deployers can call this method"
        );
    }
}
