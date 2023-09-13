use crate::*;

pub(crate) fn account_vec_to_set(
    account_vec: Vec<AccountId>,
    storage_key: StorageKey,
) -> UnorderedSet<AccountId> {
    let mut set = UnorderedSet::new(storage_key);
    for element in account_vec.iter() {
        set.insert(element);
    }
    set
}
