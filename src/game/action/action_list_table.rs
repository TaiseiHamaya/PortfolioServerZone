use super::action_list;

pub struct ActionListTable {
    action_lists: Vec<action_list::ActionList>,
}

impl ActionListTable {
    pub fn load_from_database() -> Self {
        let mut action_lists = Vec::new();
        action_lists.push(
            action_list::ActionList::new()
        );
        ActionListTable {
            action_lists
        }
    }

    pub fn get_action_list(&self, job_id: u32) -> Option<&action_list::ActionList> {
        self.action_lists.get(job_id as usize)
    }
}