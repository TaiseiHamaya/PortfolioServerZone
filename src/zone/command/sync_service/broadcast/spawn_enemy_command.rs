use crate::zone::command::CommandTrait;

use crate::{
    generated::proto_client::{PayloadEnemySpawn, Vector3},
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

        let clients = zone.gateway_clients().clients_vec();

        let message = PayloadEnemySpawn {
            id: self.enemy_id,
            name: "".to_string(),
            position: Some(Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
        };

        for mut client in clients {
            let message_clone = message.clone();
            tokio::spawn(async move {
                if let Err(e) = client.enemy_spawn(message_clone).await {
                    log::error!("Failed to send enemy spawn notification: {}", e);
                }
            });
        }
    }
}
