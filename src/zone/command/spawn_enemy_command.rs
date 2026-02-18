use super::command_trait::CommandTrait;

use crate::{net::proto, zone::zone};
use protobuf::*;

pub struct SpawnEnemyCommand {
    enemy_id: u64,
}

impl SpawnEnemyCommand {
    pub fn new(enemy_id: u64) -> Self {
        SpawnEnemyCommand { enemy_id }
    }
}

impl CommandTrait for SpawnEnemyCommand {
    fn execute(&self, zone: &mut zone::Zone) {
        // クライアント通知
        log::info!("Spawning enemy with ID {}.", self.enemy_id);
        let mut packet = proto::Packet::new();
        packet.set_category_enemy_message(proto::CategoryEnemyMessage::EnemySpawn);
        let mut body = proto::PayloadEnemySpawn::new();
        body.set_id(self.enemy_id);
        body.set_name("RedComet");
        let mut position = proto::Vector3::new();
        position.set_x(0.0);
        position.set_y(0.0);
        position.set_z(0.0);
        body.set_position(position);
        let payload = body.serialize();
        if payload.is_err() {
            log::error!(
                "Failed to serialize enemy spawn packet: {}",
                payload.unwrap_err()
            );
            return;
        }
        packet.set_payload(payload.unwrap());
        zone.players_mut().iter_mut().for_each(|(_, cluster)| {
            cluster.stack_packet(packet.clone());
        });
    }
}
