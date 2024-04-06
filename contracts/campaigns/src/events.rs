use crate::*;

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
