use crate::*;

pub type CampaignId = u64;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Campaign {
    // indexed at ID so don't need to include here
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
    pub allow_fee_avoidance: bool,
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
        owner: campaign.owner.clone(),
        name: campaign.name.clone(),
        description: campaign.description.clone(),
        cover_image_url: campaign.cover_image_url.clone(),
        recipient: campaign.recipient.clone(),
        start_ms: campaign.start_ms,
        end_ms: campaign.end_ms,
        created_ms: campaign.created_ms,
        ft_id: campaign.ft_id.clone(),
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
        target_amount: U128,
        min_amount: Option<U128>,
        max_amount: Option<U128>,
        referral_fee_basis_points: Option<u32>,
        creator_fee_basis_points: Option<u32>,
        allow_fee_avoidance: Option<bool>,
    ) -> CampaignExternal {
        let initial_storage_usage = env::storage_usage();
        let campaign_id = self.next_campaign_id;
        let campaign = Campaign {
            name,
            description,
            cover_image_url,
            recipient,
            start_ms,
            end_ms,
            owner: env::predecessor_account_id(),
            created_ms: env::block_timestamp_ms(),
            ft_id: ft_id.clone(),
            target_amount: target_amount.into(),
            min_amount: min_amount.map(|v| v.into()),
            max_amount: max_amount.map(|v| v.into()),
            total_raised_amount: 0,
            net_raised_amount: 0,
            escrow_balance: 0,
            referral_fee_basis_points: std::cmp::min(
                referral_fee_basis_points.unwrap_or(self.default_referral_fee_basis_points),
                MAX_REFERRAL_FEE_BASIS_POINTS,
            ),
            creator_fee_basis_points: std::cmp::min(
                creator_fee_basis_points.unwrap_or(self.default_creator_fee_basis_points),
                MAX_CREATOR_FEE_BASIS_POINTS,
            ),
            allow_fee_avoidance: allow_fee_avoidance.unwrap_or(false),
        };
        self.internal_insert_new_campaign_record(&campaign_id, &campaign);
        refund_deposit(initial_storage_usage);
        let formatted = format_campaign(&campaign_id, &campaign);
        log_campaign_create_event(&formatted);
        formatted
    }

    // Some old code to validate ft_id:
    //     if let Some(ft_id) = ft_id.clone() {
    //         PromiseOrValue::Promise(
    //             Promise::new(ft_id)
    //                 .function_call(
    //                     "ft_metadata".to_string(),
    //                     json!({}).to_string().into_bytes(),
    //                     0,
    //                     Gas(XCC_GAS_DEFAULT),
    //                 )
    //                 .then(
    //                     Self::ext(env::current_account_id())
    //                         .with_static_gas(Gas(XCC_GAS_DEFAULT))
    //                         .validate_ft_callback(
    //                             env::predecessor_account_id(),
    //                             &campaign_id,
    //                             &campaign,
    //                         ),
    //                 ),
    //         )
    //     } else {
    //         PromiseOrValue::Value(self.add_campaign(&campaign_id, &campaign))
    //     }
    // }

    // #[private]
    // pub fn validate_ft_callback(
    //     &mut self,
    //     caller_id: AccountId,
    //     campaign_id: &CampaignId,
    //     campaign: &Campaign,
    //     #[callback_result] call_result: Result<bool, PromiseError>,
    // ) -> CampaignExternal {
    //     if call_result.is_err() {
    //         panic!("Failed to get metadata for FT ID");
    //     } else {
    //         self.add_campaign(campaign_id, campaign)
    //     }
    // }

    // #[private]
    // pub fn add_campaign(
    //     &mut self,
    //     caller_id: AccountId,
    //     campaign_id: &CampaignId,
    //     campaign: &Campaign,
    // ) -> CampaignExternal {
    //     let initial_storage_usage = env::storage_usage();
    //     self.internal_insert_new_campaign_record(campaign_id, campaign);
    //     // refund_deposit(initial_storage_usage);
    //     let storage_used = env::storage_usage() - initial_storage_usage;
    //     let formatted = format_campaign(&campaign_id, campaign);
    //     log_campaign_create_event(&formatted);
    //     formatted
    // }

    #[payable]
    pub fn update_campaign(
        &mut self,
        campaign_id: CampaignId,
        name: Option<String>,
        description: Option<String>,
        cover_image_url: Option<String>,
        start_ms: Option<TimestampMs>,
        end_ms: Option<TimestampMs>,
        ft_id: Option<AccountId>,
        target_amount: Option<Balance>,
        max_amount: Option<Balance>,
        min_amount: Option<U128>, // Can only be provided if campaign has not started yet
        allow_fee_avoidance: Option<bool>,
        // NB: recipient cannot be updated. If incorrect recipient is specified, campaign should be deleted and recreated
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
            // ! TODO: ADD THIS BACK IN AFTER TESTING
            // assert!(
            //     campaign.start_ms > env::block_timestamp_ms(),
            //     "Cannot update start_ms once campaign has started"
            // );
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
        // Owner can change min_amount before campaign starts
        if let Some(min_amount) = min_amount {
            assert!(
                campaign.start_ms > env::block_timestamp_ms(),
                "Cannot update min_amount once campaign has started"
            );
            campaign.min_amount = Some(min_amount.into());
        }

        // Owner can change allow_fee_avoidance at any time
        if let Some(allow_fee_avoidance) = allow_fee_avoidance {
            campaign.allow_fee_avoidance = allow_fee_avoidance;
        }

        self.campaigns_by_id
            .insert(&campaign_id, &VersionedCampaign::Current(campaign.clone()));
        refund_deposit(initial_storage_usage);
        let formatted = format_campaign(&campaign_id, &campaign);
        log_campaign_update_event(&formatted);
        formatted
    }

    #[payable]
    pub fn delete_campaign(&mut self, campaign_id: CampaignId) {
        self.assert_campaign_owner(&campaign_id);
        let initial_storage_usage = env::storage_usage();
        self.internal_remove_campaign_record(campaign_id);
        refund_deposit(initial_storage_usage);
        log_campaign_delete_event(&campaign_id);
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
