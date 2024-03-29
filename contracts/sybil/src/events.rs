use crate::*;

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

/// add provider
pub(crate) fn log_add_provider_event(provider_id: &ProviderId, provider: &Provider) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "add_provider",
                "data": [
                    {
                        "provider_id": provider_id,
                        "provider": provider,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// update provider
pub(crate) fn log_update_provider_event(provider_id: &ProviderId, provider: &Provider) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "update_provider",
                "data": [
                    {
                        "provider_id": provider_id,
                        "provider": provider,
                    }
                ]
            })
        )
        .as_ref(),
    );
}
