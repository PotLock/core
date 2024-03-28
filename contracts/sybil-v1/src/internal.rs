use crate::*;

impl Contract {
    pub(crate) fn assert_at_least_one_yocto(&self) {
        assert!(
            env::attached_deposit() >= 1,
            "At least one yoctoNEAR must be attached"
        );
    }

    pub(crate) fn is_owner(&self) -> bool {
        env::predecessor_account_id() == self.owner
    }

    pub(crate) fn is_admin(&self) -> bool {
        self.admins.contains(&env::predecessor_account_id())
    }

    pub(crate) fn is_owner_or_admin(&self) -> bool {
        self.is_owner() || self.is_admin()
    }

    pub(crate) fn assert_owner(&self) {
        assert!(self.is_owner(), "Only contract owner can call this method");
        // require owner to attach at least one yoctoNEAR for security purposes
        self.assert_at_least_one_yocto();
    }

    pub(crate) fn assert_admin(&self) {
        assert!(self.is_admin(), "Only contract admin can call this method");
        // require caller to attach at least one yoctoNEAR for security purposes
        self.assert_at_least_one_yocto();
    }

    pub(crate) fn assert_owner_or_admin(&self) {
        assert!(
            self.is_owner_or_admin(),
            "Only contract owner or admin can call this method"
        );
        // require caller to attach at least one yoctoNEAR for security purposes
        self.assert_at_least_one_yocto();
    }
}
