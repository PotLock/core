use crate::*;

pub type CampaignId = u64;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Campaign {
    // indexed at ID
    pub owner: AccountId,
    pub name: String,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub recipient: AccountId,
    pub start_ms: TimestampMs,
    pub end_ms: Option<TimestampMs>,
    pub created_ms: TimestampMs,
    pub ft_id: Option<AccountId>,
    pub target_amount: Balance,
    pub min_amount: Option<Balance>,
    pub max_amount: Option<Balance>,
    pub total_raised_amount: Balance,
    pub net_raised_amount: Balance,
    pub escrow_balance: Balance,
    pub referral_fee_basis_points: u32,
    pub creator_fee_basis_points: u32,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedCampaign {
    Current(Campaign),
}

impl From<VersionedCampaign> for Campaign {
    fn from(campaign: VersionedCampaign) -> Self {
        match campaign {
            VersionedCampaign::Current(current) => current,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CampaignExternal {
    pub id: CampaignId,
    pub owner: AccountId,
    pub name: String,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub recipient: AccountId,
    pub start_ms: TimestampMs,
    pub end_ms: Option<TimestampMs>,
    pub created_ms: TimestampMs,
    pub ft_id: Option<AccountId>,
    pub target_amount: U128,
    pub min_amount: Option<U128>,
    pub max_amount: Option<U128>,
    pub total_raised_amount: U128,
    pub net_raised_amount: U128,
    pub escrow_balance: U128,
    pub referral_fee_basis_points: u32,
    pub creator_fee_basis_points: u32,
}

pub(crate) fn format_campaign(campaign_id: &CampaignId, campaign: &Campaign) -> CampaignExternal {
    CampaignExternal {
        id: campaign_id.clone(),
        owner: campaign.owner,
        name: campaign.name,
        description: campaign.description,
        cover_image_url: campaign.cover_image_url,
        recipient: campaign.recipient,
        start_ms: campaign.start_ms,
        end_ms: campaign.end_ms,
        created_ms: campaign.created_ms,
        ft_id: campaign.ft_id,
        target_amount: U128::from(campaign.target_amount),
        min_amount: campaign.min_amount.map(U128::from),
        max_amount: campaign.max_amount.map(U128::from),
        total_raised_amount: U128::from(campaign.total_raised_amount),
        net_raised_amount: U128::from(campaign.net_raised_amount),
        escrow_balance: U128::from(campaign.escrow_balance),
        referral_fee_basis_points: campaign.referral_fee_basis_points,
        creator_fee_basis_points: campaign.creator_fee_basis_points,
    }
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn create_campaign(
        &mut self,
        name: String,
        description: Option<String>,
        cover_image_url: Option<String>,
        recipient: AccountId,
        start_ms: TimestampMs,
        end_ms: Option<TimestampMs>,
        ft_id: Option<AccountId>,
        target_amount: Balance,
        min_amount: Option<Balance>,
        max_amount: Option<Balance>,
        referral_fee_basis_points: u32,
        creator_fee_basis_points: u32,
    ) -> CampaignExternal {
        let initial_storage_usage = env::storage_usage();
        let campaign_id = self.next_campaign_id;
        // TODO: VALIDATE THAT REFERRAL FEE BASIS POINTS & CREATOR FEE BASIS POINTS ARE WITHIN MAX RANGE
        let campaign = Campaign {
            name,
            description,
            cover_image_url,
            recipient,
            start_ms,
            end_ms,
            owner: env::predecessor_account_id(),
            created_ms: env::block_timestamp(),
            ft_id,
            target_amount,
            min_amount,
            max_amount,
            total_raised_amount: 0,
            net_raised_amount: 0,
            escrow_balance: 0,
            referral_fee_basis_points,
            creator_fee_basis_points,
        };
        // TODO: VALIDATE FT ID
        self.internal_insert_new_campaign_record(campaign_id, campaign);
        refund_deposit(initial_storage_usage);
        format_campaign(&campaign_id, &campaign)
    }

    pub(crate) fn internal_insert_new_campaign_record(
        &mut self,
        campaign_id: CampaignId,
        campaign: Campaign,
    ) {
        self.campaigns_by_id
            .insert(&campaign_id, &VersionedCampaign::Current(campaign));
        let mut campaign_ids_for_owner =
            self.campaign_ids_by_owner
                .get(&campaign.owner)
                .unwrap_or(UnorderedSet::new(StorageKey::CampaignIdsByOwnerInner {
                    owner_id: campaign.owner.clone(),
                }));
        campaign_ids_for_owner.insert(&campaign_id);
        self.campaign_ids_by_owner
            .insert(&campaign.owner, &campaign_ids_for_owner);
        let mut campaign_ids_for_recipient = self
            .campaign_ids_by_recipient
            .get(&campaign.recipient)
            .unwrap_or(UnorderedSet::new(StorageKey::CampaignIdsByRecipientInner {
                recipient_id: campaign.recipient.clone(),
            }));
        campaign_ids_for_recipient.insert(&campaign_id);
        self.campaign_ids_by_recipient
            .insert(&campaign.recipient, &campaign_ids_for_recipient);
    }

    pub(crate) fn internal_remove_campaign_record(&mut self, campaign_id: CampaignId) {
        let campaign = Campaign::from(
            self.campaigns_by_id
                .get(&campaign_id)
                .expect("Campaign not found"),
        );
        self.campaigns_by_id.remove(&campaign_id);
        let mut campaign_ids_for_owner = self
            .campaign_ids_by_owner
            .get(&campaign.owner)
            .expect("Campaign owner not found");
        campaign_ids_for_owner.remove(&campaign_id);
        self.campaign_ids_by_owner
            .insert(&campaign.owner, &campaign_ids_for_owner);
        let mut campaign_ids_for_recipient = self
            .campaign_ids_by_recipient
            .get(&campaign.recipient)
            .expect("Campaign recipient not found");
        campaign_ids_for_recipient.remove(&campaign_id);
        self.campaign_ids_by_recipient
            .insert(&campaign.recipient, &campaign_ids_for_recipient);
    }

    #[payable]
    pub fn update_campaign(
        &mut self,
        campaign_id: CampaignId,
        name: Option<String>,
        description: Option<String>,
        cover_image_url: Option<String>,
        recipient: Option<AccountId>,
        start_ms: Option<TimestampMs>,
        end_ms: Option<TimestampMs>,
        ft_id: Option<AccountId>,
        target_amount: Option<Balance>,
        max_amount: Option<Balance>,
    ) -> CampaignExternal {
        self.assert_campaign_owner(&campaign_id);
        let initial_storage_usage = env::storage_usage();
        let mut campaign = Campaign::from(
            self.campaigns_by_id
                .get(&campaign_id)
                .expect("Campaign not found"),
        );
        // Owner can change name, description, cover_image_url at any time
        if let Some(name) = name {
            campaign.name = name;
        }
        if let Some(description) = description {
            campaign.description = Some(description);
        }
        if let Some(cover_image_url) = cover_image_url {
            campaign.cover_image_url = Some(cover_image_url);
        }
        // Owner can change start_ms until it has passed, after that cannot be updated
        if let Some(start_ms) = start_ms {
            assert!(
                campaign.start_ms > env::block_timestamp_ms(),
                "Cannot update start_ms once campaign has started"
            );
            assert!(
                start_ms > env::block_timestamp_ms(),
                "start_ms must be in the future"
            );
            campaign.start_ms = start_ms;
        }
        // Owner can change ft_id until campaign has started
        if let Some(ft_id) = ft_id {
            assert!(
                campaign.start_ms > env::block_timestamp_ms(),
                "Cannot update ft_id once campaign has started"
            );
            campaign.ft_id = Some(ft_id);
            // TODO: VALIDATE FT ID
        }
        // Owner can change end_ms until it has passed, or until max_amount is reached, but only if there is no min_amount or once min_amount has been met (As a campaign owner I would want to be able to extend my campaign e.g. if it is doing really well, but it shouldn't be at the expense of donors whose donations are in escrow)
        if let Some(end_ms) = end_ms {
            assert!(campaign.start_ms < end_ms, "end_ms must be after start_ms");
            assert!(
                campaign.net_raised_amount <= campaign.max_amount.unwrap_or(u128::MAX),
                "Cannot edit end_ms after max_amount has been reached"
            );
            assert!(
                campaign.min_amount.is_none()
                    || campaign.net_raised_amount >= campaign.min_amount.unwrap(),
                "Cannot edit end_ms after min_amount has been reached"
            );
            campaign.end_ms = Some(end_ms);
        }
        // Owner can change target_amount until max_amount or end_ms is reached
        if let Some(target_amount) = target_amount {
            assert!(
                campaign.net_raised_amount <= campaign.max_amount.unwrap_or(u128::MAX),
                "Cannot edit target_amount after max_amount has been reached"
            );
            assert!(
                campaign.end_ms.unwrap_or(u64::MAX) > env::block_timestamp_ms(),
                "Cannot edit target_amount after end_ms has been reached"
            );
            campaign.target_amount = target_amount;
        }
        // Owner can change max_amount until it is reached, or until end_ms is reached (whichever comes first)
        if let Some(max_amount) = max_amount {
            assert!(
                campaign.net_raised_amount <= campaign.max_amount.unwrap_or(u128::MAX),
                "Cannot edit max_amount after it has been reached"
            );
            assert!(
                campaign.end_ms.unwrap_or(u64::MAX) > env::block_timestamp_ms(),
                "Cannot edit max_amount after end_ms has been reached"
            );
            campaign.max_amount = Some(max_amount);
        }
        self.campaigns_by_id
            .insert(&campaign_id, &VersionedCampaign::Current(campaign));
        refund_deposit(initial_storage_usage);
        // TODO: LOG EVENT
        format_campaign(&campaign_id, &campaign)
    }

    #[payable]
    pub fn delete_campaign(&mut self, campaign_id: CampaignId) {
        self.assert_campaign_owner(&campaign_id);
        let initial_storage_usage = env::storage_usage();
        let campaign = Campaign::from(
            self.campaigns_by_id
                .get(&campaign_id)
                .expect("Campaign not found"),
        );
        // Owner can delete campaign if it hasn't started yet (no downside to this, since no-one will have donated yet, and might have been created by accident or had second thoughts)
        assert!(
            campaign.start_ms > env::block_timestamp_ms(),
            "Cannot delete campaign once it has started"
        );
        self.internal_remove_campaign_record(campaign_id);
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn process_refunds_batch(&mut self, campaign_id: CampaignId) {
        // TODO: WIP
        let initial_storage_usage = env::storage_usage();
        let campaign = Campaign::from(
            self.campaigns_by_id
                .get(&campaign_id)
                .expect("Campaign not found"),
        );
        // Anyone can process refunds for a campaign if it has ended and min_amount has not been reached
        assert!(
            campaign.end_ms.unwrap_or(u64::MAX) < env::block_timestamp_ms(),
            "Cannot process refunds until campaign has ended"
        );
        assert!(
            campaign.net_raised_amount < campaign.min_amount.unwrap_or(u128::MAX),
            "Cannot process refunds once min_amount has been reached"
        );
        // Get escrowed donation IDs & process refunds in batches of 100
        let mut escrowed_donation_ids = self
            .escrowed_donation_ids_by_campaign_id
            .get(&campaign_id)
            .expect("No escrowed donations found for campaign");
        let mut escrowed_donation_ids_vec = escrowed_donation_ids.to_vec();
        let mut i = 0;
        let mut refunds: HashMap<AccountId, Balance> = HashMap::new();
        while i < escrowed_donation_ids_vec.len() {
            let batch = escrowed_donation_ids_vec
                .drain(i..std::cmp::min(i + 100, escrowed_donation_ids_vec.len())) // TODO: SET MAX BATCH SIZE AS UPDATEABLE VAR IN CONTRACT
                .collect::<Vec<DonationId>>();
            for donation_id in batch {
                let donation = Donation::from(
                    self.donations_by_id
                        .get(&donation_id)
                        .expect("Donation not found"),
                );
                let mut refund_amount = donation.total_amount;
                // refund total amount minus storage costs (donation record won't actually be deleted on refund)
                let storage_before = env::storage_usage();
                // temporarily remove donation record to check how much storage cost was
                self.internal_remove_donation_record(&donation);
                let storage_after = env::storage_usage();
                refund_amount -=
                    Balance::from(storage_after - storage_before) * env::storage_byte_cost();
                // add donation record back
                self.internal_insert_donation_record(&donation, true);
                // add refund amount to current balance for donor, or create new entry
                let current_balance = refunds.get(&donation.donor_id).unwrap_or(&0);
                refunds.insert(donation.donor_id, current_balance + refund_amount);
            }
        }
        // process refunds, call callback to verify refund was successful and update escrowed_donation_ids and Donation.returned
    }

    // VIEW METHODS

    pub fn get_campaign(&self, campaign_id: CampaignId) -> CampaignExternal {
        let campaign = Campaign::from(
            self.campaigns_by_id
                .get(&campaign_id)
                .expect("Campaign not found"),
        );
        format_campaign(&campaign_id, &campaign)
    }

    pub fn get_campaigns(
        &self,
        from_index: Option<u128>,
        limit: Option<u128>,
    ) -> Vec<CampaignExternal> {
        let start_index: u128 = from_index.unwrap_or_default();
        assert!(
            (self.campaigns_by_id.len() as u128) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        self.campaigns_by_id
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|(id, v)| format_campaign(&id, &Campaign::from(v)))
            .collect()
    }

    pub fn get_campaigns_by_owner(
        &self,
        owner_id: AccountId,
        from_index: Option<u128>,
        limit: Option<u128>,
    ) -> Vec<CampaignExternal> {
        let start_index: u128 = from_index.unwrap_or_default();
        let campaigns_for_owner_set = self.campaign_ids_by_owner.get(&owner_id);
        let campaigns_for_owner = if campaigns_for_owner_set.is_none() {
            vec![]
        } else {
            campaigns_for_owner_set.unwrap().to_vec()
        };
        assert!(
            (campaigns_for_owner.len() as u128) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        campaigns_for_owner
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|campaign_id| {
                format_campaign(
                    &campaign_id,
                    &Campaign::from(self.campaigns_by_id.get(&campaign_id).unwrap()),
                )
            })
            .collect()
    }

    pub fn get_campaigns_by_recipient(
        &self,
        recipient_id: AccountId,
        from_index: Option<u128>,
        limit: Option<u128>,
    ) -> Vec<CampaignExternal> {
        let start_index: u128 = from_index.unwrap_or_default();
        let campaigns_for_recipient_set = self.campaign_ids_by_recipient.get(&recipient_id);
        let campaigns_for_recipient = if campaigns_for_recipient_set.is_none() {
            vec![]
        } else {
            campaigns_for_recipient_set.unwrap().to_vec()
        };
        assert!(
            (campaigns_for_recipient.len() as u128) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        campaigns_for_recipient
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|campaign_id| {
                format_campaign(
                    &campaign_id,
                    &Campaign::from(self.campaigns_by_id.get(&campaign_id).unwrap()),
                )
            })
            .collect()
    }
}
