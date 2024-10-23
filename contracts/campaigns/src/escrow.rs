use crate::*;

/// Temporary record for refunding donations in batches. Includes an amount which may represent many donations.
#[near(serializers=[borsh, json])]
#[derive(Clone)]
pub struct TempRefundRecord {
    pub amount: Balance,
    pub escrow_balance: Balance,
    pub donations: Vec<Donation>,
}

pub type ReferrerPayouts = HashMap<AccountId, Balance>;

#[near]
impl Contract {
    /// * Process (aka move out of escrow) a batch of escrowed donations for a campaign
    /// * Can be called by anyone willing to pay the gas (max gas to avoid hitting gas limits)
    /// * Will return void without panicking if min_amount has not been reached
    pub fn process_escrowed_donations_batch(&mut self, campaign_id: CampaignId) {
        assert!(
            env::prepaid_gas() >= Gas::from_tgas(MAX_TGAS),
            "Must attach max gas to process donations"
        );
        let cp_to_process = self
            .campaigns_by_id
            .get(&campaign_id)
            .expect("Campaign not found")
            .clone();
        let campaign = Campaign::from(cp_to_process);
        assert!(
            campaign.total_raised_amount >= campaign.min_amount.unwrap_or(u128::MAX),
            "Cannot process donations until min_amount has been reached"
        );
        // Proceed with processing donations
        let escrowed_donation_ids = self
            .escrowed_donation_ids_by_campaign_id
            .get_mut(&campaign_id)
            .expect("No escrowed donations set found for campaign");
        let unescrowed_donation_ids = self
            .unescrowed_donation_ids_by_campaign_id
            .get_mut(&campaign_id)
            .expect("No unescrowed donations set found for campaign");
        // calculate totals to pay out for recipient, protocol fee, creator fee, and referral fees
        let mut recipient_total: Balance = 0; // only one recipient
        let mut protocol_fee_total: Balance = 0; // only one protocol fee recipient
        let mut creator_fee_total: Balance = 0; // only one creator fee recipient
        let mut referrer_payouts: ReferrerPayouts = HashMap::new(); // potentially multiple referral fee recipients

        // get batch of escrowed donations to process
        let mut escrowed_donation_ids_vec: Vec<DonationId> =
            escrowed_donation_ids.iter().cloned().collect();
        let donation_ids_batch = escrowed_donation_ids_vec
            .drain(0..std::cmp::min(BATCH_SIZE, escrowed_donation_ids_vec.len()))
            .collect::<Vec<DonationId>>();
        // process donation_ids_batch
        log!(
            "{}",
            format!(
                "Processing {} donations for campaign {}",
                donation_ids_batch.len(),
                campaign_id
            )
        );
        for donation_id in donation_ids_batch.iter() {
            let donation = Donation::from(
                self.donations_by_id
                    .get(&donation_id)
                    .expect("Donation not found")
                    .clone(),
            );
            // verify that donation has not been refunded (should not be in escrowed_donation_ids if it has been refunded, but just to be safe)
            if donation.returned_at_ms.is_some() {
                continue;
            }
            log!(
                "{}",
                format!(
                    "Processing donation {:#?} for campaign {:#?}",
                    donation_id, campaign_id
                )
            );
            // calculate total amount to pay out for recipient, protocol fee, creator fee, and referral fees
            // NB: there may be many referrers, but only one recipient, protocol fee recipient, and creator fee recipient
            recipient_total += donation.net_amount;
            protocol_fee_total += donation.protocol_fee;
            creator_fee_total += donation.creator_fee;
            if let Some(referrer_id) = donation.referrer_id.clone() {
                let referrer_payout = referrer_payouts.entry(referrer_id).or_insert(0);
                *referrer_payout += donation.referrer_fee.unwrap_or(0);
            }
            // remove donation from escrowed_donation_ids // TODO: revert if transfer fails
            escrowed_donation_ids.remove(&donation.id);
            // add to unescrowed_donation_ids
            unescrowed_donation_ids.insert(donation.id); // TODO: revert if transfer fails
        }
        // transfer payouts to recipients
        if recipient_total > 0 {
            log!(
                "{}",
                format!(
                    "Transferring {} to campaign recipient {} for campaign {}",
                    recipient_total, campaign.recipient, campaign_id
                )
            );
            self.internal_transfer_amount(
                recipient_total,
                campaign.recipient.clone(),
                campaign.ft_id.clone(),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::from_tgas(XCC_GAS_DEFAULT))
                    .escrowed_donations_recipient_transfer_callback(
                        recipient_total,
                        campaign.recipient.clone(),
                        campaign_id,
                        donation_ids_batch,
                        protocol_fee_total,
                        creator_fee_total,
                        referrer_payouts,
                    ),
            );
        }
    }

    #[private]
    /// Called after processing a batch of donations for a campaign, and transferring the total payout to the recipient
    pub fn escrowed_donations_recipient_transfer_callback(
        &mut self,
        amount: Balance,
        recipient: AccountId,
        campaign_id: CampaignId,
        donation_ids: Vec<DonationId>,
        protocol_fee: Balance,
        creator_fee: Balance,
        referrer_payouts: ReferrerPayouts,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        if call_result.is_err() {
            // * ERROR HANDLING
            // * Recipient amount transfer failed
            // * Revert Donation.returned_at_ms and re-insert donations into escrowed_donation_ids, remove from unescrowed_donation_ids
            // * NB: fees haven't been transferred yet, so no need to worry about those
            log!("{}", format!("Error transferring amount {:#?} to recipient {:#?} for donations {:#?} in campaign {:#?}", amount, recipient, donation_ids, campaign_id));
            let escrowed_donation_ids = self
                .escrowed_donation_ids_by_campaign_id
                .get_mut(&campaign_id)
                .expect("No escrowed donations found for campaign");
            let unescrowed_donation_ids = self
                .unescrowed_donation_ids_by_campaign_id
                .get_mut(&campaign_id)
                .expect("No unescrowed donations found for campaign");
            for donation_id in donation_ids.iter() {
                // re-insert donations into escrowed_donation_ids
                escrowed_donation_ids.insert(*donation_id);
                // remove donations from unescrowed_donation_ids
                unescrowed_donation_ids.remove(donation_id);
                // revert Donation.returned_at_ms
                let v_donation = self
                    .donations_by_id
                    .get(&donation_id)
                    .expect("Donation not found")
                    .clone();
                let mut donation = Donation::from(v_donation);
                donation.returned_at_ms = None;
                self.donations_by_id
                    .insert(donation_id.clone(), VersionedDonation::Current(donation));
            }
        } else {
            // * SUCCESS HANDLING
            log!("{}", format!(
                "Successfully transferred amount {:#?} to recipient {:#?} for donations {:#?} in campaign {:#?}",
                amount, recipient, donation_ids, campaign_id
            ));
            // log event
            log_escrow_process_event(&donation_ids);
            // send fees
            let v_campaign = self
                .campaigns_by_id
                .get(&campaign_id)
                .expect("Campaign not found")
                .clone();
            let campaign = Campaign::from(v_campaign);
            if protocol_fee > 0 {
                log!(
                    "{}",
                    format!(
                        "Transferring {} to protocol fee recipient {} for campaign {}",
                        protocol_fee, self.protocol_fee_recipient_account, campaign_id
                    )
                );
                self.internal_transfer_amount(
                    protocol_fee,
                    self.protocol_fee_recipient_account.clone(),
                    campaign.ft_id.clone(),
                )
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(Gas::from_tgas(XCC_GAS_DEFAULT))
                        .escrowed_donations_fee_transfer_callback(
                            protocol_fee,
                            self.protocol_fee_recipient_account.clone(),
                            campaign_id,
                            donation_ids.clone(),
                            FundsReceiver::Protocol,
                        ),
                );
            }
            if creator_fee > 0 {
                log!(
                    "{}",
                    format!(
                        "Transferring {} to campaign owner {} for campaign {}",
                        creator_fee, campaign.owner, campaign_id
                    )
                );
                self.internal_transfer_amount(
                    creator_fee,
                    campaign.owner.clone(),
                    campaign.ft_id.clone(),
                )
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(Gas::from_tgas(XCC_GAS_DEFAULT))
                        .escrowed_donations_fee_transfer_callback(
                            creator_fee,
                            campaign.owner.clone(),
                            campaign_id,
                            donation_ids.clone(),
                            FundsReceiver::Creator,
                        ),
                );
            }
            for (referrer_id, amount) in referrer_payouts.iter() {
                log!(
                    "{}",
                    format!(
                        "Transferring referrer payouts for campaign {:#?}: {:#?}",
                        campaign_id, referrer_payouts
                    )
                );
                self.internal_transfer_amount(*amount, referrer_id.clone(), campaign.ft_id.clone())
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(Gas::from_tgas(XCC_GAS_DEFAULT))
                            .escrowed_donations_fee_transfer_callback(
                                *amount,
                                referrer_id.clone(),
                                campaign_id,
                                donation_ids.clone(),
                                FundsReceiver::Referrer,
                            ),
                    );
            }
        }
    }

    #[private]
    pub fn escrowed_donations_fee_transfer_callback(
        &mut self,
        amount: Balance,
        recipient: AccountId,
        campaign_id: CampaignId,
        donation_ids: Vec<DonationId>,
        receiver_type: FundsReceiver,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        if call_result.is_err() {
            // * ERROR HANDLING
            // * Fee transfer failed
            // * Based on receiver_type, set relevant field on Donation (protocol_fee/referrer_fee/creator_fee) to 0
            // * NB: recipient amount has already been transferred and cannot be reverted
            log!("{}", format!(
                "Error transferring fee {:#?} to {:#?} ({:#?}) for donations {:#?} in campaign {:#?}",
                amount, recipient, receiver_type, donation_ids, campaign_id
            ));
            for donation_id in donation_ids.iter() {
                let dnm = self
                    .donations_by_id
                    .get(&donation_id)
                    .expect("Donation not found")
                    .clone();
                let mut donation = Donation::from(dnm);
                match receiver_type {
                    FundsReceiver::Protocol => {
                        donation.protocol_fee = 0;
                    }
                    FundsReceiver::Referrer => {
                        donation.referrer_fee = Some(0);
                    }
                    FundsReceiver::Creator => {
                        donation.creator_fee = 0;
                    }
                    _ => {}
                }
                self.donations_by_id
                    .insert(donation_id.clone(), VersionedDonation::Current(donation));
            }
        } else {
            // * SUCCESS HANDLING
            if receiver_type == FundsReceiver::Protocol {
                log!(
                    "{}",
                    format!(
                    "Successfully transferred amount {:#?} to recipient {:#?} for campaign {:#?}",
                    amount, recipient, campaign_id
                )
                );
            } else if receiver_type == FundsReceiver::Creator {
                log!("{}", format!(
                "Successfully transferred amount {:#?} to protocol fee recipient {:#?} for campaign {:#?}",
                amount, recipient, campaign_id
            ));
            } else if receiver_type == FundsReceiver::Referrer {
                log!(
                    "{}",
                    format!(
                    "Successfully transferred amount {:#?} to referrer {:#?} for campaign {:#?}",
                    amount, recipient, campaign_id
                )
                );
            }
        }
    }

    pub fn process_refunds_batch(&mut self, campaign_id: CampaignId) {
        // OBJECTIVES:
        // Donors must always be able to get their money out if campaign has ended and minimum amount has not been reached, and they have not been refunded yet
        // Refunds should be processed in batches to avoid hitting gas limits
        // Can be processed by anyone willing to pay the gas
        let cp = self
            .campaigns_by_id
            .get(&campaign_id)
            .expect("Campaign not found")
            .clone();
        let campaign = Campaign::from(cp);
        // Anyone can process refunds for a campaign if it has ended and min_amount has not been reached
        assert!(
            campaign.end_ms.unwrap_or(u64::MAX) < env::block_timestamp_ms(),
            "Cannot process refunds until campaign has ended"
        );
        assert!(
            campaign.total_raised_amount < campaign.min_amount.unwrap_or(u128::MAX),
            "Cannot process refunds once min_amount has been reached"
        );
        // Get escrowed donation IDs & process refunds in batches of 100
        let escrowed_donation_ids = self
            .escrowed_donation_ids_by_campaign_id
            .get(&campaign_id)
            .expect("No escrowed donations found for campaign");
        let mut escrowed_donation_ids_vec: Vec<DonationId> =
            escrowed_donation_ids.iter().cloned().collect();
        let mut refunds: HashMap<AccountId, TempRefundRecord> = HashMap::new();
        let batch = escrowed_donation_ids_vec
            .drain(0..std::cmp::min(BATCH_SIZE, escrowed_donation_ids_vec.len()))
            .collect::<Vec<DonationId>>();
        for donation_id in batch {
            let mut donation = Donation::from(
                self.donations_by_id
                    .get(&donation_id)
                    .expect("Donation not found")
                    .clone(),
            );
            // verify that donation has not already been refunded (should not be the case if it is in escrowed_donation_ids, but just to be safe)
            if donation.returned_at_ms.is_some() {
                continue;
            }
            donation.returned_at_ms = Some(env::block_timestamp_ms()); // this will be reverted if refund fails // TODO: verify that this is the case
                                                                       // refund total amount minus storage costs (since donation record won't actually be deleted on refund)
            let mut refund_amount = donation.total_amount;
            let storage_before = env::storage_usage();
            // temporarily remove donation record to check how much storage cost is (donation won't actually be deleted on refund)
            self.internal_remove_donation_record(&donation);
            let storage_after = env::storage_usage();
            refund_amount -= Balance::from(storage_after - storage_before)
                * env::storage_byte_cost().as_yoctonear();
            // add donation record back (as an escrowed donation)
            self.internal_insert_donation_record(&donation, true);
            // add refund amount to current balance for donor, or create new entry. Include donation IDs for use in callback
            let temp_refund_record =
                refunds
                    .entry(donation.donor_id.clone())
                    .or_insert(TempRefundRecord {
                        amount: 0,
                        escrow_balance: 0,
                        donations: vec![],
                    });
            temp_refund_record.amount += refund_amount;
            temp_refund_record.escrow_balance += donation.net_amount;
            temp_refund_record.donations.push(donation);
            // TODO: also remove donation from escrowed_donation_ids? (or just leave it there and update Donation.returned_at_ms)
        }
        // process refunds, call callback to verify refund was successful and update escrowed_donation_ids and Donation.returned_at_ms
        for (donor_id, refund_record) in refunds.iter() {
            self.internal_transfer_amount(
                refund_record.amount,
                donor_id.clone(),
                campaign.ft_id.clone(),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas::from_tgas(XCC_GAS_DEFAULT))
                    .transfer_refund_callback(donor_id.clone(), refund_record.clone(), campaign_id),
            );
        }
    }

    /// Verifies whether refund was successful and updates escrowed_donation_ids and Donation.returned accordingly for each donation refunded for this donor
    #[private] // Public - but only callable by env::current_account_id()
    pub fn transfer_refund_callback(
        &mut self,
        donor_id: AccountId,
        mut temp_refund_record: TempRefundRecord,
        campaign_id: CampaignId,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        if call_result.is_err() {
            // * Refund failed
            // * Log failure
            // * Revert Donation.returned_at_ms for each donation, and re-insert into escrowed_donation_ids
            log!("{}", format!(
                "Error transferring refund amount {:#?} to donor {:#?} for donations {:#?} in campaign {:#?}",
                temp_refund_record.amount, donor_id, temp_refund_record.donations, campaign_id
            ));
            for donation in temp_refund_record.donations.iter_mut() {
                donation.returned_at_ms = None;
                self.internal_insert_donation_record(&donation, true); // will add to escrowed_donation_ids
            }
        } else {
            // * Refund successful
            log!("{}", format!(
                "Successfully transferred refund amount {:#?} to donor {:#?} for donations {:#?} in campaign {:#?}",
                temp_refund_record.amount, donor_id, temp_refund_record.donations, campaign_id
            ));
            // remove donations from escrowed_donation_ids
            for donation in temp_refund_record.donations.iter() {
                self.internal_insert_donation_record(&donation, false); // will remove from escrowed_donation_ids
            }
            // remove from Campaign.escrow_balance
            let mut campaign = Campaign::from(
                self.campaigns_by_id
                    .get(&campaign_id)
                    .expect("Campaign not found")
                    .clone(),
            );
            campaign.escrow_balance -= temp_refund_record.escrow_balance;
            self.campaigns_by_id
                .insert(campaign_id, VersionedCampaign::Current(campaign));
            // NB: keeping Campaign.total_raised_amount and Campaign.net_raised_amount the same (use these as record of total donations to campaign)
            // log event
            // log_escrow_refund_event(&temp_refund_record);
        }
    }
}
