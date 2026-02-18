use super::command_trait::CommandTrait;

use crate::{game::entity::entity::Entity, net::proto, zone::zone};
use protobuf::*;

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
    fn execute(&self, zone: &mut zone::Zone) {
        log::info!(
            "Entity {} damaged entity {} for {} points.",
            self.attacker_id,
            self.target_id,
            self.damage
        );
        // ダメージ処理
        let containts_director = zone.containts_director_mut(0);
        if containts_director.is_none() {
            log::error!("ContaintsDirector not found for zone.");
            return;
        }

        let containts_director = containts_director.unwrap();
        let enemy = containts_director
            .get_enemies_mut()
            .get_mut(&self.target_id);
        if enemy.is_none() {
            log::error!("Target entity {} not found.", self.target_id);
            return;
        }

        enemy.unwrap().on_damaged(self.damage);

        // ダメージ通知パケットを作成
        let mut packet = proto::Packet::new();
        packet.set_category_sync_message(proto::CategorySyncMessage::EntityDamaged);
        let mut body: proto::PayloadEntityDamaged = proto::PayloadEntityDamaged::new();
        body.set_attacker_id(self.attacker_id);
        body.set_target_id(self.target_id);
        body.set_damage(self.damage);
        let payload = body.serialize();
        if payload.is_err() {
            log::error!(
                "Failed to serialize damage notification packet: {}",
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
