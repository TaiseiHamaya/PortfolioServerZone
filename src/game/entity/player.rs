use std::collections::HashMap;

use chrono::{self};
use nalgebra::Point3;
use rand::{self, RngExt};

use super::{
    entity::{Entity, PlayActionError, PlayActionOk},
    entity_id::EntityId,
};

use crate::game::action::{self, action_list::ActionList};

pub struct Player {
    entity_id: EntityId,

    position: Point3<f32>,
    radius: f32,
    hitpoint: i32,

    current_action_id: Option<u32>,

    action_recent_time: HashMap<u32, chrono::DateTime<chrono::Utc>>,
}

impl Entity for Player {
    fn update(&mut self) {}

    fn on_damaged(&mut self, damage: i32) -> () {
        self.hitpoint -= damage;
        if self.hitpoint < 0 {
            self.hitpoint = 0;
            return;
        }
    }

    fn position(&self) -> &Point3<f32> {
        &self.position
    }
    fn position_mut(&mut self) -> &mut Point3<f32> {
        &mut self.position
    }
    fn radius(&self) -> f32 {
        self.radius
    }
    fn entity_id(&self) -> u64 {
        self.entity_id.id()
    }

    fn play_action(
        &mut self,
        action_id: u32,
        play_utc: &chrono::DateTime<chrono::Utc>,
        action_list: &ActionList,
    ) -> Result<PlayActionOk, PlayActionError> {
        let Some(action) = action_list.get_action_by_id(action_id) else {
            // アクションがアクションリストに存在しない場合
            return Err(PlayActionError::ActionNotFoundFromList);
        };
        let action_id = if action.action_type() == action::action::ActionType::WEAPONSKILL {
            0u32
        } else {
            action_id
        };

        // recast time check
        if self
            .action_recent_time
            .get(&action_id)
            .map_or(false, |recent_time| {
                *recent_time + action.recast_time() > *play_utc
            })
        {
            return Err(PlayActionError::ActionRecastTime);
        };

        // update
        self.action_recent_time.insert(action_id, *play_utc);
        self.current_action_id = Some(action_id);

        // damage calculation (TODO)
        let mut rng = rand::rng();
        Ok(PlayActionOk::Damage(rng.random_range(900..1100)))
    }
}

impl Player {
    pub fn new(entity_id: u64, position: Point3<f32>, hitpoint: i32) -> Self {
        Player {
            entity_id: EntityId::new(entity_id),
            position,
            radius: 1.0,
            hitpoint,
            current_action_id: None,
            action_recent_time: HashMap::new(),
        }
    }
}
