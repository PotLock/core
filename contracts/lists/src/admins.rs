use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn owner_change_owner(&mut self, list_id: ListId, new_owner_id: AccountId) -> AccountId {
        self.assert_list_owner(&list_id);
        let initial_storage_usage = env::storage_usage();
        let mut list =
            ListInternal::from(self.lists_by_id.get(&list_id).expect("List does not exist"));
        let old_owner = list.owner.clone();
        list.owner = new_owner_id.clone();
        self.lists_by_id
            .insert(&list_id, &VersionedList::Current(list.clone()));
        let mut lists_for_old_owner = self
            .list_ids_by_owner
            .get(&old_owner)
            .expect("List IDs for old owner do not exist");
        lists_for_old_owner.remove(&list_id);
        self.list_ids_by_owner
            .insert(&old_owner, &lists_for_old_owner);
        let mut lists_for_new_owner =
            self.list_ids_by_owner
                .get(&new_owner_id)
                .unwrap_or(UnorderedSet::new(StorageKey::ListIdsByOwnerInner {
                    owner: new_owner_id.clone(),
                }));
        lists_for_new_owner.insert(&list_id);
        self.list_ids_by_owner
            .insert(&new_owner_id, &lists_for_new_owner);
        refund_deposit(initial_storage_usage);
        log_owner_transfer_event(list_id, new_owner_id.clone());
        new_owner_id
    }

    #[payable]
    pub fn owner_add_admins(&mut self, list_id: ListId, admins: Vec<AccountId>) -> Vec<AccountId> {
        self.assert_list_owner(&list_id);
        let initial_storage_usage = env::storage_usage();
        let mut list_admins = self
            .list_admins_by_list_id
            .get(&list_id)
            .expect("List admins do not exist");
        for admin in admins {
            list_admins.insert(&admin);
        }
        self.list_admins_by_list_id.insert(&list_id, &list_admins);
        refund_deposit(initial_storage_usage);
        log_update_admins_event(list_id, list_admins.to_vec());
        list_admins.to_vec()
    }

    #[payable]
    pub fn owner_remove_admins(
        &mut self,
        list_id: ListId,
        admins: Vec<AccountId>,
    ) -> Vec<AccountId> {
        self.assert_list_owner(&list_id);
        let initial_storage_usage = env::storage_usage();
        let mut list_admins = self
            .list_admins_by_list_id
            .get(&list_id)
            .expect("List admins do not exist");
        for admin in admins {
            list_admins.remove(&admin);
        }
        self.list_admins_by_list_id.insert(&list_id, &list_admins);
        refund_deposit(initial_storage_usage);
        log_update_admins_event(list_id, list_admins.to_vec());
        list_admins.to_vec()
    }

    #[payable]
    pub fn owner_clear_admins(&mut self, list_id: ListId) -> Vec<AccountId> {
        self.assert_list_owner(&list_id);
        let initial_storage_usage = env::storage_usage();
        let list_admins = self
            .list_admins_by_list_id
            .get(&list_id)
            .expect("List admins do not exist");
        self.list_admins_by_list_id.insert(
            &list_id,
            &UnorderedSet::new(StorageKey::ListAdminsByListIdInner { list_id }),
        );
        refund_deposit(initial_storage_usage);
        log_update_admins_event(list_id, vec![]);
        list_admins.to_vec()
    }
}
