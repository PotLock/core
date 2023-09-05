use crate::*;

impl Contract {
    pub(crate) fn assert_chef(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.chef_id,
            "Only the chef can call this method"
        );
    }
}
