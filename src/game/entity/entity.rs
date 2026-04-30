#![allow(dead_code)]

use chrono;
use nalgebra::Point3;

use crate::game::action::action_list::ActionList;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayActionOk {
    Heal(i32),
    Damage(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayActionError {
    ActionNotFoundTimer,
    ActionNotFoundFromList,
    ActionRecastTime,
}

pub trait Entity {
    fn update(&mut self) -> ();

    fn on_damaged(&mut self, damage: i32) -> ();

    fn position(&self) -> &Point3<f32>;
    fn position_mut(&mut self) -> &mut Point3<f32>;
    fn radius(&self) -> f32;
    fn entity_id(&self) -> u64;
    fn hitpoint(&self) -> i32;

    fn play_action(
        &mut self,
        action_id: u32,
        play_utc: &chrono::DateTime<chrono::Utc>,
        action_list: &ActionList,
    ) -> Result<PlayActionOk, PlayActionError>;
}
