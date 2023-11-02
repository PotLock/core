use crate::*;

/// donation
pub(crate) fn log_donation_event(donation: &Donation) {
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
