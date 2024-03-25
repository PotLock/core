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

/// create list event
pub(crate) fn log_create_list_event(list: &ListExternal) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "create_list",
                "data": [
                    {
                        "list": list
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// update list event
pub(crate) fn log_update_list_event(list: &ListInternal) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "update_list",
                "data": [
                    {
                        "list": list,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// delete list event
pub(crate) fn log_delete_list_event(list_id: ListId) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "delete_list",
                "data": [
                    {
                        "list_id": list_id,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// upvote list event
pub(crate) fn log_upvote_event(list_id: ListId, account_id: AccountId) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "upvote",
                "data": [
                    {
                        "list_id": list_id,
                        "account_id": account_id,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// remove upvote list event
pub(crate) fn log_remove_upvote_event(list_id: ListId, account_id: AccountId) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "remove_upvote",
                "data": [
                    {
                        "list_id": list_id,
                        "account_id": account_id,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// update (add or remove) admins event
pub(crate) fn log_update_admins_event(list_id: ListId, admins: Vec<AccountId>) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "add_admins",
                "data": [
                    {
                        "list_id": list_id,
                        "admins": admins,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// owner transfer event
pub(crate) fn log_owner_transfer_event(list_id: ListId, new_owner_id: AccountId) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "owner_transfer",
                "data": [
                    {
                        "list_id": list_id,
                        "new_owner_id": new_owner_id,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// create registration event
pub(crate) fn log_create_registration_event(registration: &RegistrationExternal) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "create_registration",
                "data": [
                    {
                        "registration": registration,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// update registration event
pub(crate) fn log_update_registration_event(registration: &RegistrationExternal) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "update_registration",
                "data": [
                    {
                        "registration": registration,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// delete registration event
pub(crate) fn log_delete_registration_event(registration_id: RegistrationId) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "delete_registration",
                "data": [
                    {
                        "registration_id": registration_id,
                    }
                ]
            })
        )
        .as_ref(),
    );
}
