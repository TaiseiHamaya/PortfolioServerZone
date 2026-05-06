use crate::zone::command::CommandTrait;

use crate::{
    generated::proto_server::{BroadcastStream, PayloadEnemySpawn, Vector3, broadcast_stream},
    zone::zone,
};

pub struct SpawnEnemyCommand {
    enemy_id: u64,
}

impl SpawnEnemyCommand {
    pub fn new(enemy_id: u64) -> Self {
        SpawnEnemyCommand { enemy_id }
    }
}

impl CommandTrait for SpawnEnemyCommand {
    fn execute(self: Box<Self>, zone: &mut zone::Zone) {
        // クライアント通知
        log::info!("Spawning enemy with ID {}.", self.enemy_id);

        let message = PayloadEnemySpawn {
            enemy_type_id: self.enemy_id,
            entity_id: zone.next_entity_id(),
            position: Some(Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
        };

        let message = BroadcastStream {
            payload: Some(broadcast_stream::Payload::EnemySpawn(message)),
        };

        zone.send_broadcast_message(message);
    }
}
