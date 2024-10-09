use crate::*;

/// * `Donation` is the data structure that is stored within the contract.
/// * *NB: recipient & ft_id are stored in the Campaign struct.*
#[near(serializers=[borsh, json])]
#[derive(Clone, Debug)]
pub struct Donation {
    /// Unique identifier for the donation
    pub id: DonationId,
    /// Campaign ID
    pub campaign_id: CampaignId,
    /// ID of the donor               
    pub donor_id: AccountId,
    /// Amount donated         
    pub total_amount: u128,
    /// Net amount, after all fees & storage costs
    pub net_amount: u128,
    /// Optional message from the donor          
    pub message: Option<String>,
    /// Timestamp when the donation was made
    pub donated_at_ms: TimestampMs,
    /// Protocol fee
    pub protocol_fee: u128,
    /// Referrer ID
    pub referrer_id: Option<AccountId>,
    /// Referrer fee
    pub referrer_fee: Option<u128>,
    /// Creator fee
    pub creator_fee: u128,
    /// Whether (and when) the donation was returned to sender
    pub returned_at_ms: Option<TimestampMs>,
    // TODO: add paid_at_ms?
}

#[near(serializers=[borsh])]
#[derive(Clone)]
pub enum VersionedDonation {
    Current(Donation),
}

impl From<VersionedDonation> for Donation {
    fn from(donation: VersionedDonation) -> Self {
        match donation {
            VersionedDonation::Current(current) => current,
        }
    }
}

/// Ephemeral-only (used in views)
#[near(serializers=[json])]
#[derive(Clone)]
pub struct DonationExternal {
    /// Unique identifier for the donation
    pub id: DonationId,
    /// Campaign ID
    pub campaign_id: CampaignId,
    /// ID of the donor               
    pub donor_id: AccountId,
    /// Amount donated         
    pub total_amount: U128,
    /// Net amount, after all fees & storage costs
    pub net_amount: U128,
    /// FT ID (if None, it's a native NEAR donation)
    pub ft_id: Option<AccountId>,
    /// Optional message from the donor          
    pub message: Option<String>,
    /// Timestamp when the donation was made
    pub donated_at_ms: TimestampMs,
    /// Protocol fee
    pub protocol_fee: U128,
    /// Referrer ID
    pub referrer_id: Option<AccountId>,
    /// Referrer fee
    pub referrer_fee: Option<U128>,
    /// Creator fee
    pub creator_fee: U128,
    /// Whether the donation was returned to sender
    pub returned_at_ms: Option<TimestampMs>,
    /// Whether the donation is currently in escrow
    pub is_in_escrow: bool,
    /// ID of the account receiving the donation  
    pub recipient_id: AccountId,
}

#[near(serializers=[json])]
#[derive(Debug)]
pub struct FtReceiverMsg {
    pub campaign_id: CampaignId,
    pub referrer_id: Option<AccountId>,
    pub message: Option<String>,
    pub bypass_protocol_fee: Option<bool>,
    pub bypass_creator_fee: Option<bool>,
}

#[near(serializers=[json])]
#[derive(Debug, PartialEq)]
pub enum FundsReceiver {
    Recipient,
    Protocol,
    Referrer,
    Creator,
}

#[near]
impl Contract {
    /// FT equivalent of donate, for use with FTs that implement NEP-144
    pub fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let ft_id = env::predecessor_account_id();
        let msg_json: FtReceiverMsg = near_sdk::serde_json::from_str(&msg)
            .expect("Invalid msg string. Must implement FtReceiverMsg.");
        log!("{}", format!(
            "Campaign ID {:?}, Referrer ID {:?}, Amount {}, Message {:?}, ByPass Protocol Fee {:?}, ByPass Creator Fee {:?}",
            msg_json.campaign_id, msg_json.referrer_id, amount.0, msg_json.message, msg_json.bypass_protocol_fee, msg_json.bypass_creator_fee
        ));

        self.assert_campaign_live(&msg_json.campaign_id);
        // fetch campaign
        let versioned_cp = self
            .campaigns_by_id
            .get(&msg_json.campaign_id)
            .expect("Campaign not found")
            .clone();
        let campaign = Campaign::from(versioned_cp);

        // verify that ft_id is correct for this campaign
        assert_eq!(
            campaign.ft_id.clone(),
            Some(ft_id.clone()),
            "FT ID {} is not allowed for this campaign. Expected {}.",
            ft_id,
            campaign.ft_id.unwrap_or("near".parse().unwrap())
        );

        // calculate amounts
        let (protocol_fee, referrer_fee, creator_fee, amount_after_fees) = self
            .calculate_fees_and_remainder(
                amount.0,
                &campaign,
                msg_json.referrer_id.clone(),
                msg_json.bypass_protocol_fee,
                msg_json.bypass_creator_fee,
            );

        // compose Donation record
        let donation = Donation {
            id: self.next_donation_id,
            campaign_id: msg_json.campaign_id.clone(),
            donor_id: sender_id.clone(),
            total_amount: amount.0,
            net_amount: amount_after_fees,
            message: msg_json.message,
            donated_at_ms: env::block_timestamp_ms(),
            protocol_fee,
            referrer_id: msg_json.referrer_id.clone(),
            referrer_fee,
            creator_fee,
            returned_at_ms: None,
        };
        self.handle_donation(donation, campaign);

        // Return amount of unused tokens, as per NEP-144 standard
        PromiseOrValue::Value(U128(0))
    }

    #[payable]
    pub fn donate(
        &mut self,
        campaign_id: CampaignId,
        message: Option<String>,
        referrer_id: Option<AccountId>,
        bypass_protocol_fee: Option<bool>,
        bypass_creator_fee: Option<bool>,
    ) -> PromiseOrValue<DonationExternal> {
        /*
        DONATION SCENARIOS
        1. Campaign is not live (before start or after end, or max_amount reached), donate call fails
        2. Campaign is live, min_amount present & has not been reached, donation is accepted but not paid out (goes into escrow)
        3. Campaign is live, no min_amount or min_amount has been reached, donation is accepted and paid out
         */
        self.assert_campaign_live(&campaign_id);
        let v_campaign = self
            .campaigns_by_id
            .get(&campaign_id)
            .expect("Campaign not found")
            .clone();
        let campaign = Campaign::from(v_campaign);
        // calculate amounts
        let amount = env::attached_deposit();
        let (protocol_fee, referrer_fee, creator_fee, amount_after_fees) = self
            .calculate_fees_and_remainder(
                amount.as_yoctonear(),
                &campaign,
                referrer_id.clone(),
                bypass_protocol_fee,
                bypass_creator_fee,
            );

        // compose Donation record
        let donation = Donation {
            id: self.next_donation_id,
            campaign_id,
            donor_id: env::predecessor_account_id(),
            total_amount: amount.as_yoctonear(),
            net_amount: amount_after_fees,
            message,
            donated_at_ms: env::block_timestamp_ms(),
            protocol_fee,
            referrer_id,
            referrer_fee,
            creator_fee,
            returned_at_ms: None,
        };
        self.handle_donation(donation.clone(), campaign);
        PromiseOrValue::Value(self.format_donation(&donation))
    }

    /// * Increments self.next_donation_id
    /// * Inserts donation record & ID into appropriate mappings
    /// * Asserts that storage cost is covered
    /// * If campaign.total_amount >= campaign.min_amount, transfers donation; otherwise, donation is escrowed
    pub(crate) fn handle_donation(&mut self, mut donation: Donation, mut campaign: Campaign) {
        let initial_storage_usage = env::storage_usage();
        self.next_donation_id += 1;
        // if min_amount present & has not been reached, donation is accepted but not paid out (goes into escrow)
        let should_escrow = campaign.total_raised_amount < campaign.min_amount.unwrap_or(0);
        self.internal_insert_donation_record(&donation, should_escrow);

        if campaign.ft_id.is_some() {
            // verify and update storage balance for FT donation
            self.verify_and_update_storage_balance(
                donation.donor_id.clone(),
                initial_storage_usage,
            );
        } else {
            // assert that donation after fees > storage cost
            let required_deposit = calculate_required_storage_deposit(initial_storage_usage);
            require!(
                donation.net_amount > required_deposit,
                format!(
                    "Must attach {} yoctoNEAR to cover storage", // TODO: update this amount to reflect fees
                    required_deposit
                )
            );

            // update net_amount with storage taken out
            donation.net_amount = donation.net_amount - required_deposit;
            self.donations_by_id
                .insert(donation.id, VersionedDonation::Current(donation.clone()));
        }

        if should_escrow {
            log!(
                "{}",
                format!(
                    "Donation {} (ft_id {:?}) accepted but not paid out (escrowed) for campaign {}",
                    donation.net_amount, campaign.ft_id, donation.campaign_id
                )
            );
            // update campaign (raised_amount & escrow_balance)
            campaign.total_raised_amount += donation.total_amount;
            campaign.net_raised_amount += donation.net_amount;
            campaign.escrow_balance += donation.net_amount;
            self.campaigns_by_id.insert(
                donation.campaign_id,
                VersionedCampaign::Current(campaign.clone()),
            );
            // log event
            log_escrow_insert_event(&self.format_donation(&donation));
        } else {
            // transfer donation
            log!(
                "{}",
                format!(
                    "Transferring donation {} to {}",
                    donation.net_amount,
                    campaign.recipient.clone()
                )
            );
            self.handle_transfer(
                self.format_donation(&donation),
                FundsReceiver::Recipient,
                donation.net_amount.into(),
                campaign.recipient.clone(),
            );
            // * NB: fees will be transferred in transfer_funds_callback after successful transfer to recipient
        }
    }

    pub(crate) fn internal_insert_donation_record(&mut self, donation: &Donation, escrow: bool) {
        // add to donations-by-id mapping
        self.donations_by_id
            .insert(donation.id, VersionedDonation::Current(donation.clone()));

        // insert into appropriate donations-by-campaign mapping, according to whether donation is escrowed or not
        if escrow {
            // insert into escrowed set
            self.escrowed_donation_ids_by_campaign_id
                .get_mut(&donation.campaign_id)
                .map(|v| v.insert(donation.id));
            // ensure that donation is not in unescrowed or returned sets
            self.unescrowed_donation_ids_by_campaign_id
                .get_mut(&donation.campaign_id)
                .map(|v| v.remove(&donation.id));
            self.returned_donation_ids_by_campaign_id
                .get_mut(&donation.campaign_id)
                .map(|v| v.remove(&donation.id));
        } else {
            // insert into unescrowed set
            self.unescrowed_donation_ids_by_campaign_id
                .get_mut(&donation.campaign_id)
                .map(|v| v.insert(donation.id));
            // ensure that donation is not in escrowed or returned sets
            self.escrowed_donation_ids_by_campaign_id
                .get_mut(&donation.campaign_id)
                .map(|v| v.remove(&donation.id));
            self.returned_donation_ids_by_campaign_id
                .get_mut(&donation.campaign_id)
                .map(|v| v.remove(&donation.id));
        }

        // insert into donations-by-donor mapping
        self.donation_ids_by_donor_id
            .get_mut(&donation.donor_id)
            .unwrap_or(&mut IterableSet::new(
                StorageKey::DonationIdsByDonorIdInner {
                    donor_id: donation.donor_id.clone(),
                },
            ))
            .insert(donation.id);
    }

    pub(crate) fn internal_remove_donation_record(&mut self, donation: &Donation) {
        // remove from donations-by-id mapping
        self.donations_by_id.remove(&donation.id);

        // remove from donations-by-campaign mappings
        self.escrowed_donation_ids_by_campaign_id
            .get_mut(&donation.campaign_id)
            .expect("Campaign not found")
            .remove(&donation.id);

        self.unescrowed_donation_ids_by_campaign_id
            .get_mut(&donation.campaign_id)
            .expect("Campaign not found")
            .remove(&donation.id);

        // remove from donations-by-donor mapping
        self.donation_ids_by_donor_id
            .get_mut(&donation.donor_id)
            .expect("Donor not found")
            .remove(&donation.id);
    }

    // GETTERS

    pub fn get_donations(
        &self,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<DonationExternal> {
        let start_index: u128 = from_index.unwrap_or_default();
        assert!(
            (self.donations_by_id.len() as u128) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        self.donations_by_id
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|(_, v)| self.format_donation(&Donation::from(v.clone())))
            .collect()
    }

    pub fn get_donation_by_id(&self, donation_id: DonationId) -> Option<DonationExternal> {
        self.donations_by_id
            .get(&donation_id)
            .map(|v| self.format_donation(&Donation::from(v.clone())))
    }

    pub fn get_donatio(&self, campaign_id: CampaignId) -> Vec<u64> {
        // self.escrowed_donation_ids_by_campaign_id.len()
        let es_set = self
            .escrowed_donation_ids_by_campaign_id
            .get(&campaign_id)
            .expect("go home..");

        es_set.iter().cloned().collect()
    }

    pub fn get_donations_for_campaign(
        &self,
        campaign_id: CampaignId,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<DonationExternal> {
        let start_index: u128 = from_index.unwrap_or_default();
        assert!(
            (self.donations_by_id.len() as u128) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        let escrowed_donation_ids_by_campaign_set =
            self.escrowed_donation_ids_by_campaign_id.get(&campaign_id);
        let escrowed_donation_ids_vec = if let Some(ref escrowed_donation_ids_by_campaign_set) =
            escrowed_donation_ids_by_campaign_set
        {
            escrowed_donation_ids_by_campaign_set
                .iter()
                .cloned()
                .collect()
        } else {
            vec![]
        };

        log!(
            "{}",
            format!(
                "Escrowed donations for campaign {:?}: {:?}",
                escrowed_donation_ids_by_campaign_set, escrowed_donation_ids_vec
            )
        );

        let unescrowed_donation_ids_by_campaign_set = self
            .unescrowed_donation_ids_by_campaign_id
            .get(&campaign_id);
        let unescrowed_donation_ids_vec = if let Some(unescrowed_donation_ids_by_campaign_set) =
            unescrowed_donation_ids_by_campaign_set
        {
            unescrowed_donation_ids_by_campaign_set
                .iter()
                .cloned()
                .collect()
        } else {
            vec![]
        };

        // combine both vecs
        let mut donation_ids_by_campaign = escrowed_donation_ids_vec;
        donation_ids_by_campaign.extend(unescrowed_donation_ids_vec);

        // return formatted donations
        donation_ids_by_campaign
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|donation_id| {
                self.format_donation(&Donation::from(
                    self.donations_by_id.get(&donation_id).unwrap().clone(),
                ))
            })
            .collect()
    }

    pub fn get_donations_for_donor(
        &self,
        donor_id: AccountId,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<DonationExternal> {
        let start_index: u128 = from_index.unwrap_or_default();
        // TODO: ADD BELOW BACK IN
        // assert!(
        //     (self.donations_by_id.len() as u128) >= start_index,
        //     "Out of bounds, please use a smaller from_index."
        // );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        let donation_ids_by_donor_set = self.donation_ids_by_donor_id.get(&donor_id);
        if let Some(donation_ids_by_donor_set) = donation_ids_by_donor_set {
            donation_ids_by_donor_set
                .iter()
                .skip(start_index as usize)
                .take(limit)
                .map(|donation_id| {
                    self.format_donation(&Donation::from(
                        self.donations_by_id.get(&donation_id).unwrap().clone(),
                    ))
                })
                .collect()
        } else {
            vec![]
        }
    }

    pub(crate) fn format_donation(&self, donation: &Donation) -> DonationExternal {
        let cp_donation = self
            .campaigns_by_id
            .get(&donation.campaign_id)
            .expect("Campaign not found")
            .clone();
        let campaign = Campaign::from(cp_donation);
        DonationExternal {
            id: donation.id,
            campaign_id: donation.campaign_id.clone(),
            donor_id: donation.donor_id.clone(),
            recipient_id: campaign.recipient.clone(),
            total_amount: U128(donation.total_amount),
            net_amount: U128(donation.net_amount),
            ft_id: campaign.ft_id.clone(),
            message: donation.message.clone(),
            donated_at_ms: donation.donated_at_ms,
            protocol_fee: U128(donation.protocol_fee),
            referrer_id: donation.referrer_id.clone(),
            referrer_fee: donation.referrer_fee.map(|v| U128(v)),
            creator_fee: U128(donation.creator_fee),
            returned_at_ms: donation.returned_at_ms,
            is_in_escrow: self
                .escrowed_donation_ids_by_campaign_id
                .get(&donation.campaign_id)
                .map(|v| v.contains(&donation.id))
                .unwrap(),
        }
    }

    pub(crate) fn unformat_donation(&self, donation: &DonationExternal) -> Donation {
        Donation {
            id: donation.id,
            campaign_id: donation.campaign_id.clone(),
            donor_id: donation.donor_id.clone(),
            total_amount: donation.total_amount.0,
            net_amount: 0,
            message: donation.message.clone(),
            donated_at_ms: donation.donated_at_ms,
            protocol_fee: donation.protocol_fee.0,
            referrer_id: donation.referrer_id.clone(),
            referrer_fee: donation.referrer_fee.map(|v| v.0),
            creator_fee: donation.creator_fee.0,
            returned_at_ms: donation.returned_at_ms,
        }
    }
}
