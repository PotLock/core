use crate::*;

#[near]
impl Contract {
    pub(crate) fn assert_at_least_one_yocto(&self) {
        assert!(
            env::attached_deposit().as_yoctonear() >= 1,
            "At least one yoctoNEAR must be attached"
        );
    }

    pub(crate) fn is_caller_contract_owner(&self) -> bool {
        env::predecessor_account_id() == self.owner
    }

    pub(crate) fn assert_contract_owner(&self) {
        assert!(self.is_caller_contract_owner(), "Owner-only action");
        // require owner to attach at least one yoctoNEAR for security purposes
        self.assert_at_least_one_yocto();
    }

    pub(crate) fn assert_contract_admin_or_greater(&self) {
        assert!(
            self.is_caller_contract_owner() || self.admins.contains(&env::predecessor_account_id()),
            "Admin-only action"
        );
        // require admin to attach at least one yoctoNEAR for security purposes
        self.assert_at_least_one_yocto();
    }

    pub(crate) fn assert_campaign_owner(&self, campaign_id: &CampaignId) {
        let bc = self
            .campaigns_by_id
            .get(campaign_id)
            .expect("campaign notfound");
        let campaign = Campaign::from(bc.clone());
        assert_eq!(
            env::predecessor_account_id(),
            campaign.owner,
            "Owner-only action"
        );
    }

    /// Asserts that the campaign is live (before start or after end, or next_raised_amount >= max_amount)
    pub(crate) fn assert_campaign_live(&self, campaign_id: &CampaignId) {
        let campaign = Campaign::from(
            self.campaigns_by_id
                .get(campaign_id)
                .expect("Campaign not found")
                .clone(),
        );
        assert!(
            campaign.start_ms <= env::block_timestamp_ms(),
            "Campaign has not started yet"
        );
        if let Some(end_ms) = campaign.end_ms {
            assert!(
                end_ms > env::block_timestamp_ms(),
                "Campaign has already ended"
            );
        }
        if let Some(max_amount) = campaign.max_amount {
            assert!(
                campaign.net_raised_amount < max_amount,
                "Campaign has reached max amount"
            );
        }
    }

    /// Transfers specified amount to specified recipient. If ft_id is Some, transfers in FT, otherwise in NEAR.
    pub(crate) fn internal_transfer_amount(
        &self,
        amount: Balance,
        recipient: AccountId,
        ft_id: Option<AccountId>,
    ) -> Promise {
        if let Some(ft_id) = ft_id.clone() {
            // if ft_id is Some, send in FT
            let ft_transfer_args = json!({ "receiver_id": recipient, "amount": U128(amount) })
                .to_string()
                .into_bytes();

            Promise::new(ft_id).function_call(
                "ft_transfer".to_string(),
                ft_transfer_args,
                NearToken::from_yoctonear(ONE_YOCTO),
                Gas::from_tgas(XCC_GAS_DEFAULT),
            )
        } else {
            Promise::new(recipient).transfer(NearToken::from_yoctonear(amount))
        }
    }

    pub(crate) fn internal_insert_new_campaign_record(
        &mut self,
        campaign_id: &CampaignId,
        campaign: &Campaign,
    ) {
        // Insert campaign record
        self.campaigns_by_id
            .insert(*campaign_id, VersionedCampaign::Current(campaign.clone()));

        // flush keys
        self.campaigns_by_id.flush();

        let campaign_by_owner_set: &mut IterableSet<u64> =
        self.campaign_ids_by_owner
            .entry(campaign.owner.clone())
            .or_insert_with(|| IterableSet::new(StorageKey::CampaignIdsByOwnerInner {
                owner_id: campaign.owner.clone(),
            }));
        campaign_by_owner_set.insert(*campaign_id);
        campaign_by_owner_set.flush(); // Flush the inner IterableSet
        self.campaign_ids_by_owner.flush(); // Flush the outer mapping

        let campaign_by_recipient_set: &mut IterableSet<u64> =
        self.campaign_ids_by_recipient
            .entry(campaign.recipient.clone())
            .or_insert_with(|| IterableSet::new(StorageKey::CampaignIdsByRecipientInner {
                recipient_id: campaign.recipient.clone(),
            }));
        campaign_by_recipient_set.insert(*campaign_id);
        campaign_by_recipient_set.flush(); // Flush the inner IterableSet
        self.campaign_ids_by_recipient.flush(); // Flush the outer mapping


        let mut escrowed_by_camp_id = IterableSet::new(StorageKey::EscrowedDonationIdsByCampaignIdInner {
            campaign_id: campaign_id.clone(),
        });
        escrowed_by_camp_id.flush();
        self.escrowed_donation_ids_by_campaign_id
            .insert(*campaign_id, escrowed_by_camp_id);
        self.escrowed_donation_ids_by_campaign_id.flush();

        let mut unescrowed_camp_id = IterableSet::new(StorageKey::UnescrowedDonationIdsByCampaignIdInner {
            campaign_id: campaign_id.clone(),
        });

        unescrowed_camp_id.flush();

        self.unescrowed_donation_ids_by_campaign_id.insert(
            *campaign_id,
            unescrowed_camp_id,
        );

        self.unescrowed_donation_ids_by_campaign_id.flush();

        let mut returned_dona_id_set = IterableSet::new(StorageKey::ReturnedDonationIdsByCampaignIdInner {
            campaign_id: campaign_id.clone(),
        });

        returned_dona_id_set.flush();

        self.returned_donation_ids_by_campaign_id.insert(
            *campaign_id,
            returned_dona_id_set,
        );

        self.returned_donation_ids_by_campaign_id.flush();
    }

    /// * Removes a campaign and all records of its ID from storage
    /// * Panics if campaign has started or has donations
    pub(crate) fn internal_remove_campaign_record(&mut self, campaign_id: CampaignId) {
        let campaign = Campaign::from(
            self.campaigns_by_id
                .get(&campaign_id)
                .expect("Campaign not found")
                .clone(),
        );
    
        // Cannot delete campaign if it has started
        assert!(
            campaign.start_ms > env::block_timestamp_ms(),
            "Cannot delete campaign once it has started"
        );
    
        // Cannot delete campaign if it has donations
        let donations_for_campaign = self.get_donations_for_campaign(campaign_id, None, None);
        assert!(
            donations_for_campaign.is_empty(),
            "Cannot delete campaign with donations"
        );
    
        // Remove campaign record
        self.campaigns_by_id.remove(&campaign_id);
        self.campaigns_by_id.flush(); // Flush after removing campaign
    
        // Remove campaign ID from owner's and recipient's lists
        if let Some(owner_campaigns) = self.campaign_ids_by_owner.get_mut(&campaign.owner) {
            owner_campaigns.remove(&campaign_id);
            owner_campaigns.flush(); // Flush owner's campaigns list
        }
    
        if let Some(recipient_campaigns) = self.campaign_ids_by_recipient.get_mut(&campaign.recipient) {
            recipient_campaigns.remove(&campaign_id);
            recipient_campaigns.flush(); // Flush recipient's campaigns list
        }
    
        // Remove donation ID lists for campaign
        self.escrowed_donation_ids_by_campaign_id.remove(&campaign_id);
        self.escrowed_donation_ids_by_campaign_id.flush(); // Flush escrowed donations list
    
        self.unescrowed_donation_ids_by_campaign_id.remove(&campaign_id);
        self.unescrowed_donation_ids_by_campaign_id.flush(); // Flush unescrowed donations list
    
        self.returned_donation_ids_by_campaign_id.remove(&campaign_id);
        self.returned_donation_ids_by_campaign_id.flush(); // Flush returned donations list
    }
    
}
