use crate::*;

impl Contract {
    pub(crate) fn assert_at_least_one_yocto(&self) {
        assert!(
            env::attached_deposit() >= 1,
            "At least one yoctoNEAR must be attached"
        );
    }

    pub(crate) fn assert_contract_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Owner-only action"
        );
        // require owner to attach at least one yoctoNEAR for security purposes
        self.assert_at_least_one_yocto();
    }

    pub(crate) fn assert_campaign_owner(&self, campaign_id: &CampaignId) {
        let campaign = Campaign::from(
            self.campaigns_by_id
                .get(campaign_id)
                .expect("Campaign not found"),
        );
        assert_eq!(
            env::predecessor_account_id(),
            campaign.owner,
            "Owner-only action"
        );
    }

    /// Asserts that the campaign is live (before start or after end, or max_amount reached)
    pub(crate) fn assert_campaign_live(&self, campaign_id: &CampaignId) {
        let campaign = Campaign::from(
            self.campaigns_by_id
                .get(campaign_id)
                .expect("Campaign not found"),
        );
        assert!(
            campaign.start_ms <= env::block_timestamp(),
            "Campaign has not started yet"
        );
        if let Some(end_ms) = campaign.end_ms {
            assert!(
                end_ms > env::block_timestamp(),
                "Campaign has already ended"
            );
        }
        if let Some(max_amount) = campaign.max_amount {
            assert!(
                campaign.raised_amount < max_amount,
                "Campaign has reached max amount"
            );
        }
    }
}
