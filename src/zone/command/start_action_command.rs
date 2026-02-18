use super::command_trait::CommandTrait;

use crate::{net::proto, zone::zone};
use protobuf::*;

pub struct StartActionCommand {
    id: u64,
    action_id: u32,
}

impl StartActionCommand {
    pub fn new(id: u64, action_id: u32) -> Self {
        StartActionCommand { id, action_id }
    }
}

impl CommandTrait for StartActionCommand {
    fn execute(&self, zone: &mut zone::Zone) {
        log::info!("Entity {} begins action {}.", self.id, self.action_id);
        let mut packet = proto::Packet::new();
        packet.set_category_sync_message(proto::CategorySyncMessage::PlayAction);
        let mut body = proto::PayloadPlayAction::new();
        body.set_id(self.id);
        body.set_action_id(self.action_id);
        let payload = body.serialize();
        if payload.is_err() {
            return;
        }
        packet.set_payload(payload.unwrap());
        // クライアントに通知
        zone.players_mut().iter_mut().for_each(|(_, cluster)| {
            cluster.stack_packet(packet.clone());
        });
    }
}
