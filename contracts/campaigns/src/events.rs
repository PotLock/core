use crate::*;

/// update config

pub(crate) fn log_config_update_event(config: &Config) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "config_update",
                "data": [
                    {
                        "config": config,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// campaign creation
pub(crate) fn log_campaign_create_event(campaign: &CampaignExternal) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "campaign_create",
                "data": [
                    {
                        "campaign": campaign,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// campaign update
pub(crate) fn log_campaign_update_event(campaign: &CampaignExternal) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "campaign_update",
                "data": [
                    {
                        "campaign": campaign,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// campaign deletion
pub(crate) fn log_campaign_delete_event(campaign_id: &CampaignId) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "campaign_delete",
                "data": [
                    {
                        "campaign_id": campaign_id,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// donation
pub(crate) fn log_donation_event(donation: &DonationExternal) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "donation",
                "data": [
                    {
                        "donation": donation,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// escrowed donation insert
pub(crate) fn log_escrow_insert_event(donation: &DonationExternal) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "escrow_insert",
                "data": [
                    {
                        "donation": donation,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// escrowed donation refund
pub(crate) fn log_escrow_refund_event(temp_refund_record: &TempRefundRecord) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "escrow_refund",
                "data": [
                    {
                        "amount": temp_refund_record.amount,
                //         "donations": temp_refund_record.donations.iter().map(|donation| {
                //     json!({
                //         "id": donation.id,
                //         "total_amount": donation.total_amount,
                //         "net_amount": donation.net_amount,
                //         "protocol_fee": donation.protocol_fee,
                //         "creator_fee": donation.creator_fee,
                //         "referrer_fee": donation.referrer_fee,
                //     })
                // }).collect::<Vec<_>>(),
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// escrowed donation process
pub(crate) fn log_escrow_process_event(donation_ids: &Vec<DonationId>) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "escrow_process",
                "data": [
                    {
                        "donation_ids": donation_ids,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// source metadata update
pub(crate) fn log_set_source_metadata_event(source_metadata: &ContractSourceMetadata) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "set_source_metadata",
                "data": [
                    {
                        "source_metadata": source_metadata,
                    }
                ]
            })
        )
        .as_ref(),
    );
}
