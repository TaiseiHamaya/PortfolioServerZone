use chrono;
use nalgebra::Point3;

use super::entity::{Entity, PlayActionError, PlayActionOk};
use super::entity_id::EntityId;

pub struct Enemy {
    id: EntityId,
    name: String,
    position: Point3<f32>,
    hitpoint: i32,
    radius: f32,

    hate_list: Vec<u64>,
}

impl Enemy {
    pub fn new(id: u64, position: Point3<f32>, radius: f32) -> Self {
        Enemy {
            id: EntityId::new(id),
            name: format!("Enemy_{}", id),
            hitpoint: 10000,
            position,
            radius,
            hate_list: Vec::new(),
        }
    }

    pub fn get_hate_list(&self) -> &Vec<u64> {
        &self.hate_list
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }
}

impl Entity for Enemy {
    fn update(&mut self) {
        // Update logic for the player can be added here
    }

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

    fn play_action(
        &mut self,
        action_id: u32,
        play_utc: &chrono::DateTime<chrono::Utc>,
    ) -> Result<PlayActionOk, PlayActionError> {
        Ok(PlayActionOk::Damage(0))
    }
}
