use nalgebra::Point3;

use crate::zone::command::CommandTrait;

use crate::{game::entity::entity::Entity, zone::zone};

#[allow(dead_code)]
pub struct SyncTransformCommand {
    entity_id: u64,
    position: Point3<f32>,
    timestamp: u64,
}

impl SyncTransformCommand {
    pub fn new(entity_id: u64, position: Point3<f32>, timestamp: u64) -> Self {
        SyncTransformCommand {
            entity_id,
            position,
            timestamp,
        }
    }
}

impl CommandTrait for SyncTransformCommand {
    fn execute(self: Box<Self>, zone: &mut zone::Zone) {
        // クライアント通知
        log::info!("Syncing transform for entity with ID {}.", self.entity_id);

        let Some(player) = zone.players_mut().get_mut(&self.entity_id) else {
            log::warn!(
                "Player with ID {} not found for transform sync.",
                self.entity_id
            );
            return;
        };

        *player.player_mut().position_mut() = self.position;
    }
}
