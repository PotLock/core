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

#[near_bindgen]
impl Contract {
    /// Panics if exact set of providers already exists in another group. Ignores empty set.
    pub(crate) fn assert_valid_providers_vec(&self, providers: &Vec<ProviderId>) {
        if providers.len() > 0 {
            for (group_id, _group) in self.groups_by_id.iter() {
                if let Some(provider_ids) = self.provider_ids_for_group.get(&group_id) {
                    if provider_ids.to_vec() == *providers {
                        env::panic_str("Group with the same providers already exists");
                    }
                }
            }
        }
    }
}
