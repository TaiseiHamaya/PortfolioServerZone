use chrono;
use nalgebra::Point3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayActionOk {
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
    fn id(&self) -> u64;

    fn play_action(
        &mut self,
        action_id: u32,
        play_utc: &chrono::DateTime<chrono::Utc>,
    ) -> Result<PlayActionOk, PlayActionError>;
}
