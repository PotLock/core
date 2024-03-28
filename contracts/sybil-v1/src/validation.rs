use crate::*;

pub(crate) fn assert_valid_provider_name(name: &str) {
    assert!(
        name.len() <= MAX_PROVIDER_NAME_LENGTH,
        "Provider name is too long"
    );
}

pub(crate) fn assert_valid_provider_description(description: &str) {
    assert!(
        description.len() <= MAX_PROVIDER_DESCRIPTION_LENGTH,
        "Provider description is too long"
    );
}

pub(crate) fn assert_valid_provider_gas(gas: &u64) {
    assert!(
        gas > &0 && gas <= &MAX_GAS as &u64,
        "Provider gas is too high, must be greater than zero and less than or equal to {}",
        MAX_GAS
    );
}

pub(crate) fn assert_valid_provider_external_url(external_url: &str) {
    assert!(
        external_url.len() <= MAX_PROVIDER_EXTERNAL_URL_LENGTH,
        "Provider external URL is too long"
    );
}

pub(crate) fn assert_valid_provider_icon_url(icon_url: &str) {
    assert!(
        icon_url.len() <= MAX_PROVIDER_ICON_URL_LENGTH,
        "Provider icon URL is too long"
    );
}

pub(crate) fn assert_valid_provider_tag(tag: &str) {
    assert!(tag.len() <= MAX_TAG_LENGTH, "Tag is too long");
}

pub(crate) fn assert_valid_provider_tags(tags: &[String]) {
    assert!(
        tags.len() <= MAX_TAGS_PER_PROVIDER,
        "Too many tags for provider"
    );
    for tag in tags {
        assert_valid_provider_tag(tag);
    }
}
