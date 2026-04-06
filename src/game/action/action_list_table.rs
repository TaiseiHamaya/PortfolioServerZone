use std::sync::OnceLock;

use super::action_list;

#[derive(Debug)]
pub struct ActionListTable {
    action_lists: Vec<action_list::ActionList>,
}

impl ActionListTable {
    pub fn load_from_database() {
        let mut action_lists = Vec::new();
        action_lists.push(action_list::ActionList::new());
        ACTION_LIST_TABLE
            .set(ActionListTable { action_lists })
            .expect("Failed to set ActionListTable");
    }

    pub fn get_action_list(&self, job_id: usize) -> Option<&action_list::ActionList> {
        self.action_lists.get(job_id)
    }
}

pub static ACTION_LIST_TABLE: OnceLock<ActionListTable> = OnceLock::new();
