use crate::zone::command::CommandTrait;

use crate::{generated::proto_client::PayloadEntityDamaged, zone::zone};

#[allow(dead_code)]
pub struct DamagedEntityCommand {
    attacker_id: u64,
    target_id: u64,
    damage: i32,
}

impl DamagedEntityCommand {
    pub fn new(attacker_id: u64, target_id: u64, damage: i32) -> Self {
        DamagedEntityCommand {
            attacker_id,
            target_id,
            damage,
        }
    }
}

impl CommandTrait for DamagedEntityCommand {
    fn execute(self: Box<Self>, zone: &mut zone::Zone) {
        log::info!(
            "Entity {} damaged entity {} for {} points.",
            self.attacker_id,
            self.target_id,
            self.damage
        );
        // ダメージ処理
        let Some(entity) = zone.entity_mut(&self.target_id) else {
            log::error!("Target entity {} not found in zone.", self.target_id);
            return;
        };

        entity.on_damaged(self.damage);

        let clients = zone.gateway_clients().clients_vec();

        let message = PayloadEntityDamaged {
            entity_id: self.target_id,
            damage: self.damage,
        };

        tokio::spawn(async move {
            for mut client in clients {
                if let Err(e) = client.entity_damaged(message.clone()).await {
                    log::error!("Failed to send damage notification: {}", e);
                }
            }
        });
    }
}
