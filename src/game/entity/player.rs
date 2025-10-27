use std::collections::HashMap;

use chrono;
use nalgebra::Point3;
use rand::{self, Rng};

use super::{
    entity::{Entity, PlayActionError, PlayActionOk},
    entity_id::EntityId,
};

use crate::game::action;

pub struct Player<'action_list> {
    id: EntityId,
    position: Point3<f32>,
    radius: f32,
    hitpoint: i32,

    current_action_id: Option<u32>,

    action_list: &'action_list action::action_list::ActionList,
    action_recent_time: HashMap<u32, chrono::DateTime<chrono::Utc>>,
}

impl Entity for Player<'_> {
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
    fn id(&self) -> u64 {
        self.id.id()
    }

    fn play_action(&mut self, action_id: u32, play_utc: &chrono::DateTime<chrono::Utc>) -> Result<PlayActionOk, PlayActionError> {
        let recent_time = self.action_recent_time.get_mut(&action_id);
        // アクションがタイマーに存在しない場合
        if recent_time.is_none() {
            return Err(PlayActionError::ActionNotFoundTimer);
        }
        let recent_time = recent_time.unwrap();
        let action = self.action_list.get_action_by_id(action_id);
        if action.is_none() {
            // アクションがアクションリストに存在しない場合
            return Err(PlayActionError::ActionNotFoundFromList);
        }
        let action = action.unwrap();
        // クールタイム中の場合
        if *recent_time + action.recast_time() > *play_utc {
            return Err(PlayActionError::ActionRecastTime);
        }

        *recent_time = *play_utc;
        let mut rng = rand::rng();
        Ok(PlayActionOk::Damage(rng.random_range(900..1100)))
    }
}

impl<'action_list> Player<'action_list> {
    pub fn new(
        id: u64,
        position: Point3<f32>,
        hitpoint: i32,
        action_list: &'action_list action::action_list::ActionList,
    ) -> Self {
        Player {
            id: EntityId::new(id),
            position,
            radius: 1.0,
            hitpoint,
            current_action_id: None,
            action_list,
            action_recent_time: HashMap::new(),
        }
    }
}
