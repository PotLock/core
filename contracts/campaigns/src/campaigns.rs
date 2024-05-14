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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TempRefundRecord {
    pub amount: Balance,
    pub donations: Vec<Donation>,
    // pub donation_ids: Vec<DonationId>,
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
        referral_fee_basis_points: Option<u32>, // TODO: Make sure to pay this out
        creator_fee_basis_points: Option<u32>,  // TODO: do something with these
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
            created_ms: env::block_timestamp_ms(),
            ft_id,
            target_amount: target_amount.into(),
            min_amount: min_amount.map(|v| v.into()),
            max_amount: max_amount.map(|v| v.into()),
            total_raised_amount: 0,
            net_raised_amount: 0,
            escrow_balance: 0,
            referral_fee_basis_points: referral_fee_basis_points
                .unwrap_or(self.default_referral_fee_basis_points),
            creator_fee_basis_points: creator_fee_basis_points
                .unwrap_or(self.default_creator_fee_basis_points),
        };
        // TODO: VALIDATE FT ID
        self.internal_insert_new_campaign_record(campaign_id, &campaign);
        refund_deposit(initial_storage_usage);
        let formatted = format_campaign(&campaign_id, &campaign);
        log_campaign_create_event(&formatted);
        formatted
    }

    pub(crate) fn internal_insert_new_campaign_record(
        &mut self,
        campaign_id: CampaignId,
        campaign: &Campaign,
    ) {
        self.campaigns_by_id
            .insert(&campaign_id, &VersionedCampaign::Current(campaign.clone()));
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
        recipient: Option<AccountId>, // TODO: determine whether to include recipient
        start_ms: Option<TimestampMs>,
        end_ms: Option<TimestampMs>,
        ft_id: Option<AccountId>,
        target_amount: Option<Balance>,
        max_amount: Option<Balance>,
        min_amount: Option<U128>, // Can only be provided if campaign has not started yet
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
            // TODO: ADD THIS BACK IN AFTER TESTING
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
        log_campaign_delete_event(&campaign_id);
    }

    #[payable]
    pub fn process_donations_for_campaign(&mut self, campaign_id: CampaignId) {
        // process all escrowed donations for a campaign
        // anyone can call, as long as they pay the gas (max gas to avoid hitting gas limits)
        // should not continue if min_amount has not been reached
        let attached_gas = env::prepaid_gas();
        assert!(
            attached_gas >= Gas(MAX_TGAS),
            "Must attach max gas to process donations"
        );
        let campaign = Campaign::from(
            self.campaigns_by_id
                .get(&campaign_id)
                .expect("Campaign not found"),
        );
        if campaign.net_raised_amount >= campaign.min_amount.unwrap_or(u128::MAX) {
            // process all escrowed donations for this campaign
            // calculate totals for recipient, protocol fee, creator fee, and referral fee
            let mut escrowed_donation_ids = self
                .escrowed_donation_ids_by_campaign_id
                .get(&campaign_id)
                .expect("No escrowed donations found for campaign");
            let mut unescrowed_donation_ids = self
                .unescrowed_donation_ids_by_campaign_id
                .get(&campaign_id)
                .expect("No unescrowed donations found for campaign");
            // calculate totals to pay out for recipient, protocol fee, creator fee, and referral fees
            // one recipient, protocol fee recipient, and creator fee recipient
            // multiple referral fee recipients
            let mut payouts: HashMap<AccountId, Balance> = HashMap::new();
            for donation_id in escrowed_donation_ids.to_vec().iter() {
                let donation = Donation::from(
                    self.donations_by_id
                        .get(&donation_id)
                        .expect("Donation not found"),
                );
                // verify that donation has not been refunded (should not be the case if it is in escrowed_donation_ids, but just to be safe)
                if donation.returned {
                    continue;
                }
                // calculate total amount to pay out for recipient, protocol fee, creator fee, and referral fees
                // let total_amount = donation.total_amount;
                // add to payouts
                let recipient_payout = payouts.entry(campaign.recipient.clone()).or_insert(0);
                *recipient_payout += donation.net_amount;
                let protocol_fee_payout = payouts
                    .entry(self.protocol_fee_recipient_account.clone())
                    .or_insert(0);
                *protocol_fee_payout += donation.protocol_fee;
                if let Some(referrer_id) = donation.referrer_id.clone() {
                    let referrer_payout = payouts.entry(referrer_id).or_insert(0);
                    *referrer_payout += donation.referrer_fee.unwrap_or(0);
                }
                // TODO: handle creator fee
                // remove donation from escrowed_donation_ids (TODO: consider how to handle if transfer fails)
                escrowed_donation_ids.remove(&donation.id);
                // add to unescrowed_donation_ids
                unescrowed_donation_ids.insert(&donation.id);
            }
            // insert updated escrowed_donation_ids
            self.escrowed_donation_ids_by_campaign_id
                .insert(&campaign_id, &escrowed_donation_ids);
            // insert updated unescrowed_donation_ids
            self.unescrowed_donation_ids_by_campaign_id
                .insert(&campaign_id, &unescrowed_donation_ids);
            // transfer payouts to recipients
            log!(format!(
                "Transferring payouts for campaign {:#?}: {:#?}",
                campaign_id, payouts
            ));
            // TODO: figure out how to handle callbacks / error handling during transfers
            for (recipient_id, amount) in payouts.iter() {
                // if ft_id is None, donate in NEAR
                if campaign.ft_id.is_none() {
                    Promise::new(recipient_id.clone()).transfer(*amount);
                    // TODO: add callback back in
                    // .then(
                    //     Self::ext(env::current_account_id())
                    //         .with_static_gas(Gas(XCC_GAS_DEFAULT))
                    //         .transfer_donation_callback(
                    //             donor_id.clone(),
                    //             *amount,
                    //             campaign_id,
                    //             donations_vec.clone(),
                    //         ),
                    // );
                } else {
                    // if ft_id is Some, donate in FT
                    let ft_transfer_args =
                        json!({ "receiver_id": recipient_id, "amount": U128(*amount) })
                            .to_string()
                            .into_bytes();

                    Promise::new(campaign.ft_id.clone().unwrap()).function_call(
                        "ft_transfer".to_string(),
                        ft_transfer_args,
                        ONE_YOCTO,
                        Gas(XCC_GAS_DEFAULT),
                    );
                    // TODO: add callback back in
                    // .then(Self::ext(env::current_account_id()).with_callback(
                    //     env::predecessor_account_id(),
                    //     recipient_id.clone(),
                    //     *amount,
                    //     campaign_id,
                    //     escrowed_donation_ids_vec.clone(),
                    // ));
                }
            }
        } else {
            log!(format!(
                "Cannot process donations until min_amount has been reached for campaign {:#?}",
                campaign_id
            ));
            return;
        }
    }
    // #[payable]
    // pub fn process_donations_batch(&mut self, campaign_id: CampaignId) {
    //     // process all escrowed donations for a campaign in batches of 100
    //     // should assert that min_amount has been reached before processing donations

    //     // verify max gas
    //     assert!(
    //         env::prepaid_gas() >= Gas(MAX_TGAS),
    //         "Must attach max gas to process donations batch"
    //     );

    //     // let initial_storage_usage = env::storage_usage();
    //     let campaign = Campaign::from(
    //         self.campaigns_by_id
    //             .get(&campaign_id)
    //             .expect("Campaign not found"),
    //     );
    //     assert!(
    //         campaign.net_raised_amount >= campaign.min_amount.unwrap_or(u128::MAX),
    //         "Cannot process donations until min_amount has been reached"
    //     );
    //     let mut escrowed_donation_ids = self
    //         .escrowed_donation_ids_by_campaign_id
    //         .get(&campaign_id)
    //         .expect("No escrowed donations found for campaign");
    //     let mut escrowed_donation_ids_vec = escrowed_donation_ids.to_vec();
    //     // let mut total_raised_amount = campaign.total_raised_amount;
    //     // let mut net_raised_amount = campaign.net_raised_amount;
    //     // let mut escrow_balance = campaign.escrow_balance;
    //     let mut donations: HashMap<AccountId, Balance> = HashMap::new();
    //     let mut donations_vec: Vec<Donation> = vec![];
    //     let batch_size = 100;
    //     let batch = escrowed_donation_ids_vec
    //         .drain(0..std::cmp::min(batch_size, escrowed_donation_ids_vec.len()))
    //         .collect::<Vec<DonationId>>();
    //     for donation_id in batch {
    //         let donation = Donation::from(
    //             self.donations_by_id
    //                 .get(&donation_id)
    //                 .expect("Donation not found"),
    //         );
    //         // verify that donation has not been refunded (should not be the case if it is in escrowed_donation_ids, but just to be safe)
    //         if donation.returned {
    //             continue;
    //         }
    //         // add donation amount to current balance for donor, or create new entry
    //         let current_balance = donations.get(&donation.donor_id).unwrap_or(&0);
    //         donations.insert(
    //             donation.donor_id.clone(),
    //             current_balance + donation.total_amount,
    //         );
    //         donations_vec.push(donation.clone());
    //         // pre-emptively remove from escrowed_donation_ids, will add back in in callback if transfer fails
    //         escrowed_donation_ids.remove(&donation.id);
    //     }
    //     // insert updated escrowed_donation_ids
    //     self.escrowed_donation_ids_by_campaign_id
    //         .insert(&campaign_id, &escrowed_donation_ids);
    //     // process donations, call callback to verify donation was successful and update escrowed_donation_ids and Donation.processed
    //     for (donor_id, amount) in donations.iter() {
    //         // if ft_id is None, donate in NEAR
    //         if campaign.ft_id.is_none() {
    //             Promise::new(donor_id.clone()).transfer(*amount).then(
    //                 Self::ext(env::current_account_id())
    //                     .with_static_gas(Gas(XCC_GAS_DEFAULT))
    //                     .transfer_donation_callback(
    //                         donor_id.clone(),
    //                         *amount,
    //                         campaign_id,
    //                         donations_vec.clone(),
    //                     ),
    //             );
    //         } else {
    //             // if ft_id is Some, donate in FT
    //             let ft_transfer_args = json!({ "receiver_id": donor_id, "amount": U128(*amount) })
    //                 .to_string()
    //                 .into_bytes();

    //             Promise::new(campaign.ft_id.clone().unwrap())
    //                 .function_call(
    //                     "ft_transfer".to_string(),
    //                     ft_transfer_args,
    //                     ONE_YOCTO,
    //                     Gas(XCC_GAS_DEFAULT),
    //                 )
    //                 .then(
    //                     Self::ext(env::current_account_id())
    //                         .with_static_gas(Gas(XCC_GAS_DEFAULT))
    //                         .transfer_donation_callback(
    //                             donor_id.clone(),
    //                             *amount,
    //                             campaign_id,
    //                             donations_vec.clone(),
    //                         ),
    //                 );
    //         }
    //     }
    // }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn transfer_donation_callback(
        &mut self,
        donor_id: AccountId,
        amount: Balance,
        campaign_id: CampaignId,
        donations: Vec<Donation>,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        if call_result.is_err() {
            // donation processing failed
            log!(format!(
                "Error transferring donation amount {:#?} to donor {:#?} for donations {:#?} in campaign {:#?}",
                amount, donor_id, donations, campaign_id
            ));
            // re-insert donations into escrowed_donation_ids
            let mut escrowed_donation_ids = self
                .escrowed_donation_ids_by_campaign_id
                .get(&campaign_id)
                .expect("No escrowed donations found for campaign");
        } else {
            // donation successful
            log!(format!(
                "Successfully transferred donation amount {:#?} to donor {:#?} for donations {:#?} in campaign {:#?}",
                amount, donor_id, donations, campaign_id
            ));
            // remove donations from escrowed_donation_ids
            let mut escrowed_donation_ids = self
                .escrowed_donation_ids_by_campaign_id
                .get(&campaign_id)
                .expect("No escrowed donations found for campaign");
            for donation in donations.iter() {
                escrowed_donation_ids.remove(&donation.id);
                // let donation = Donation::from(
                //     self.donations_by_id
                //         .get(&donation.id)
                //         .expect("Donation not found"),
                // );
                // self.internal_insert_donation_record(&donation, false);
            }
            self.escrowed_donation_ids_by_campaign_id
                .insert(&campaign_id, &escrowed_donation_ids);
            // update Campaign.total_raised_amount, Campaign.net_raised_amount, Campaign.escrow_balance
            let mut campaign = Campaign::from(
                self.campaigns_by_id
                    .get(&campaign_id)
                    .expect("Campaign not found"),
            );
            campaign.total_raised_amount += amount;
            campaign.net_raised_amount += amount;
            campaign.escrow_balance -= amount;
            self.campaigns_by_id
                .insert(&campaign_id, &VersionedCampaign::Current(campaign));
            // log event
            for donation in donations.iter() {
                log_donation_event(&self.format_donation(donation));
            }
        }
    }

    #[payable]
    pub fn process_refunds_batch(&mut self, campaign_id: CampaignId) {
        // TODO: WIP
        // OBJECTIVES:
        // Donors must always be able to get their money out if campaign has ended and minimum amount has not been reached, and they have not been refunded yet
        // Refunds should be processed in batches to avoid hitting gas limits
        // Can be processed by anyone willing to pay the gas
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
        let escrowed_donation_ids = self
            .escrowed_donation_ids_by_campaign_id
            .get(&campaign_id)
            .expect("No escrowed donations found for campaign");
        let mut escrowed_donation_ids_vec = escrowed_donation_ids.to_vec();
        let mut refunds: HashMap<AccountId, TempRefundRecord> = HashMap::new();
        // while i < escrowed_donation_ids_vec.len() {
        let batch_size = 100; // TODO: SET BATCH SIZE (or max batch size) AS UPDATEABLE VAR IN CONTRACT
        let batch = escrowed_donation_ids_vec
            // .drain(i..std::cmp::min(i + 100, escrowed_donation_ids_vec.len()))
            .drain(0..std::cmp::min(batch_size, escrowed_donation_ids_vec.len()))
            .collect::<Vec<DonationId>>();
        for donation_id in batch {
            let donation = Donation::from(
                self.donations_by_id
                    .get(&donation_id)
                    .expect("Donation not found"),
            );
            // verify that donation has not been refunded yet (should not be the case if it is in escrowed_donation_ids, but just to be safe)
            if donation.returned {
                continue;
            }
            // refund total amount minus storage costs (since donation record won't actually be deleted on refund)
            let mut refund_amount = donation.total_amount;
            let storage_before = env::storage_usage();
            // temporarily remove donation record to check how much storage cost was
            self.internal_remove_donation_record(&donation);
            let storage_after = env::storage_usage();
            refund_amount -=
                Balance::from(storage_after - storage_before) * env::storage_byte_cost();
            // add donation record back
            self.internal_insert_donation_record(&donation, true);
            // add refund amount to current balance for donor, or create new entry
            // let current_balance = refunds.get(&donation.donor_id).unwrap_or(&0);
            // refunds.insert(donation.donor_id, current_balance + refund_amount);
            let temp_refund_record =
                refunds
                    .entry(donation.donor_id.clone())
                    .or_insert(TempRefundRecord {
                        amount: 0,
                        donations: vec![],
                    });
            temp_refund_record.amount += refund_amount;
            temp_refund_record.donations.push(donation);
        }
        // }
        // process refunds, call callback to verify refund was successful and update escrowed_donation_ids and Donation.returned
        for (donor_id, refund_record) in refunds.iter() {
            // if ft_id is None, refund in NEAR
            if campaign.ft_id.is_none() {
                Promise::new(donor_id.clone())
                    .transfer(refund_record.amount)
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(Gas(XCC_GAS_DEFAULT))
                            .transfer_refund_callback(
                                donor_id.clone(),
                                refund_record.clone(),
                                campaign_id,
                            ),
                    );
            } else {
                // if ft_id is Some, refund in FT
                let ft_transfer_args =
                    json!({ "receiver_id": donor_id, "amount": U128(refund_record.amount) })
                        .to_string()
                        .into_bytes();

                Promise::new(campaign.ft_id.clone().unwrap())
                    .function_call(
                        "ft_transfer".to_string(),
                        ft_transfer_args,
                        ONE_YOCTO,
                        Gas(XCC_GAS_DEFAULT),
                    )
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(Gas(XCC_GAS_DEFAULT))
                            .transfer_refund_callback(
                                donor_id.clone(),
                                refund_record.clone(),
                                campaign_id,
                            ),
                    );
            }
        }
    }

    /// Verifies whether refund was successful and updates escrowed_donation_ids and Donation.returned accordingly for each donation refunded for this donor
    #[private] // Public - but only callable by env::current_account_id()
    pub fn transfer_refund_callback(
        &mut self,
        donor_id: AccountId,
        temp_refund_record: TempRefundRecord,
        campaign_id: CampaignId,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        if call_result.is_err() {
            // refund failed
            log!(format!(
                "Error transferring refund amount {:#?} to donor {:#?} for donations {:#?} in campaign {:#?}",
                temp_refund_record.amount, donor_id, temp_refund_record.donations, campaign_id
            ));
            // TODO: consider panicking here
            // Nothing else to do here
        } else {
            // refund successful
            log!(format!(
                "Successfully transferred refund amount {:#?} to donor {:#?} for donations {:#?} in campaign {:#?}",
                temp_refund_record.amount, donor_id, temp_refund_record.donations, campaign_id
            ));
            // remove donations from escrowed_donation_ids & update Donation.returned to true
            let mut escrowed_donation_ids = self
                .escrowed_donation_ids_by_campaign_id
                .get(&campaign_id)
                .expect("No escrowed donations found for campaign");
            for donation in temp_refund_record.donations.iter() {
                escrowed_donation_ids.remove(&donation.id);
                let mut donation = Donation::from(
                    self.donations_by_id
                        .get(&donation.id)
                        .expect("Donation not found"),
                );
                donation.returned = true;
                self.internal_insert_donation_record(&donation, false);
            }
            self.escrowed_donation_ids_by_campaign_id
                .insert(&campaign_id, &escrowed_donation_ids);
            // remove from Campaign.esrow_balance
            let mut campaign = Campaign::from(
                self.campaigns_by_id
                    .get(&campaign_id)
                    .expect("Campaign not found"),
            );
            campaign.escrow_balance -= temp_refund_record.amount;
            self.campaigns_by_id
                .insert(&campaign_id, &VersionedCampaign::Current(campaign));
            // NB: keeping Campaign.total_raised_amount and Campaign.net_raised_amount the same (use these as record of total donations to campaign)
            // log event
            log_escrow_refund_event(&temp_refund_record);
        }
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
