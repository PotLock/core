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

/// transfer owner
pub(crate) fn log_transfer_owner_event(new_owner: &AccountId) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "transfer_owner",
                "data": [
                    {
                        "new_owner": new_owner,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// update admins
pub(crate) fn log_update_admins_event(admins: &Vec<AccountId>) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "update_admins",
                "data": [
                    {
                        "admins": admins,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// add or update provider
pub(crate) fn log_add_or_update_provider_event(provider: &ProviderExternal) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "add_or_update_provider",
                "data": [
                    {
                        "provider": provider,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// add stamp
pub(crate) fn log_add_stamp_event(stamp: &StampExternal) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "add_stamp",
                "data": [
                    {
                        "stamp": stamp,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// delete stamp
pub(crate) fn log_delete_stamp_event(stamp_id: &StampId) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "delete_stamp",
                "data": [
                    {
                        "stamp_id": stamp_id,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// add or update group
pub(crate) fn log_add_or_update_group_event(group: &GroupExternal) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "add_or_update_group",
                "data": [
                    {
                        "group": group,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// delete group
pub(crate) fn log_delete_group_event(group_name: &String) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "delete_group",
                "data": [
                    {
                        "group_name": group_name,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// update default providers
pub(crate) fn log_update_default_providers_event(default_providers: Vec<ProviderId>) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "update_default_providers",
                "data": [
                    {
                        "default_providers": default_providers,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// update default human threshold
pub(crate) fn log_update_default_human_threshold_event(default_human_threshold: u32) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "update_default_human_threshold",
                "data": [
                    {
                        "default_human_threshold": default_human_threshold,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// blacklist account
pub(crate) fn log_blacklist_accounts_event(accounts: &Vec<AccountId>, reason: &Option<String>) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "blacklist_account",
                "data": [
                    {
                        "accounts": accounts,
                        "reason": reason,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// unblacklist account
pub(crate) fn log_unblacklist_accounts_event(accounts: &Vec<AccountId>) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "unblacklist_account",
                "data": [
                    {
                        "accounts": accounts,
                    }
                ]
            })
        )
        .as_ref(),
    );
}
