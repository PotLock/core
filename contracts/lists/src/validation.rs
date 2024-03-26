use crate::*;

pub(crate) fn assert_valid_list_name(name: &str) {
    assert!(
        name.len() <= MAX_LIST_NAME_LENGTH,
        "Provider name is too long"
    );
}

pub(crate) fn assert_valid_list_description(description: &str) {
    assert!(
        description.len() <= MAX_LIST_DESCRIPTION_LENGTH,
        "Provider description is too long"
    );
}

pub(crate) fn assert_valid_url(url: &str) {
    assert!(url.starts_with("https://"), "Invalid URL");
}
