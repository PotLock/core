use crate::*;

impl Contract {
    pub(crate) fn assert_at_least_one_yocto(&self) {
        assert!(
            env::attached_deposit() >= 1,
            "At least one yoctoNEAR must be attached"
        );
    }

    pub(crate) fn is_owner(&self, account_id: Option<&AccountId>) -> bool {
        account_id.unwrap_or(&env::predecessor_account_id()) == &self.owner
    }

    pub(crate) fn is_admin(&self, account_id: Option<&AccountId>) -> bool {
        self.admins
            .contains(&account_id.unwrap_or(&env::predecessor_account_id()))
    }

    pub(crate) fn is_owner_or_admin(&self, account_id: Option<&AccountId>) -> bool {
        self.is_owner(account_id) || self.is_admin(account_id)
    }

    pub(crate) fn assert_owner(&self) {
        assert!(
            self.is_owner(None),
            "Only contract owner can call this method"
        );
        // require owner to attach at least one yoctoNEAR for security purposes
        self.assert_at_least_one_yocto();
    }

    pub(crate) fn assert_admin_or_greater(&self) {
        assert!(
            self.is_owner_or_admin(None),
            "Only contract admin or owner can call this method"
        );
        // require caller to attach at least one yoctoNEAR for security purposes
        self.assert_at_least_one_yocto();
    }

    pub(crate) fn is_chef(&self, account_id: Option<&AccountId>) -> bool {
        if let Some(chef) = self.chef.get() {
            account_id.unwrap_or(&env::predecessor_account_id()) == &chef
        } else {
            false
        }
    }

    /// Asserts that caller is, at minimum, a chef (admin or owner also allowed)
    pub(crate) fn assert_chef_or_greater(&self) {
        assert!(
            self.is_chef(None) || self.is_admin(None) || self.is_owner(None),
            "Only chef, admin or owner can call this method"
        );
        // require caller to attach at least one yoctoNEAR for security purposes
        self.assert_at_least_one_yocto();
    }

    pub(crate) fn assert_round_closed(&self) {
        assert!(
            env::block_timestamp_ms() >= self.public_round_end_ms,
            "Round is still open"
        );
    }

    pub(crate) fn assert_round_not_closed(&self) {
        assert!(
            env::block_timestamp_ms() < self.public_round_end_ms,
            "Round is closed"
        );
    }

    pub(crate) fn assert_approved_application(&self, project_id: &ProjectId) {
        assert!(
            self.approved_application_ids.contains(project_id),
            "Approved application does not exist"
        );
    }

    pub(crate) fn assert_cooldown_period_complete(&self) {
        if let Some(cooldown_end_ms) = self.cooldown_end_ms.get() {
            assert!(
                cooldown_end_ms < env::block_timestamp_ms(),
                "Cooldown period is not over"
            );
        } else {
            panic!("Cooldown period is not set");
        }
    }

    pub(crate) fn is_application_period_open(&self) -> bool {
        let block_timestamp_ms = env::block_timestamp_ms();
        block_timestamp_ms >= self.application_start_ms
            && block_timestamp_ms < self.application_end_ms
    }

    pub(crate) fn assert_application_period_open(&self) {
        assert!(
            self.is_application_period_open(),
            "Application period is not open"
        );
    }

    pub(crate) fn assert_round_active(&self) {
        assert!(self.is_round_active(), "Public round is not active");
    }

    pub(crate) fn assert_max_projects_not_reached(&self) {
        assert!(
            self.approved_application_ids.len() < self.max_projects.into(),
            "Max projects reached"
        );
    }
}
