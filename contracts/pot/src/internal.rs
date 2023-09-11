use crate::*;

impl Contract {
    pub(crate) fn assert_chef(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.chef_id,
            "Only the chef can call this method"
        );
    }

    pub(crate) fn assert_round_closed(&self) {
        assert!(
            env::block_timestamp_ms() >= self.end_time,
            "Round is still open"
        );
    }

    pub(crate) fn assert_round_open(&self) {
        assert!(env::block_timestamp_ms() < self.end_time, "Round is closed");
    }

    pub(crate) fn assert_approved_application(&self, application_id: &ApplicationId) {
        let application_exists = self.approved_application_ids.contains(application_id);
        assert!(application_exists, "Application does not exist");
    }

    pub(crate) fn assert_cooldown_period_complete(&self) {
        assert!(
            self.cooldown_end_ms.is_some()
                && self.cooldown_end_ms.unwrap() < env::block_timestamp_ms(),
            "Cooldown period is not over"
        );
    }

    pub(crate) fn assert_application_period_open(&self) {
        let block_timestamp_ms = env::block_timestamp_ms();
        assert!(
            block_timestamp_ms >= self.application_start_ms
                && block_timestamp_ms < self.application_end_ms,
            "Application period is closed"
        );
    }
}
