use super::action;

pub struct ActionList {
    actions: Vec<action::EntityAction>,
}

impl ActionList {
    pub fn new() -> Self {
        let mut actions = Vec::new();
        actions.push(action::EntityAction::new());
        ActionList {
            actions
        }
    }

    pub fn get_action_by_id(&self, action_id: u32) -> Option<&action::EntityAction> {
        self.actions.get(action_id as usize)
    }
}