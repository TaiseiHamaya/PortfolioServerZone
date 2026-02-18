use super::command_trait::CommandTrait;

use crate::{net::proto, zone::zone};
use protobuf::*;

pub struct ChatBroadcastCommand {
    id: u64,
    message: String,
}

impl ChatBroadcastCommand {
    pub fn new(id: u64, message: String) -> Self {
        ChatBroadcastCommand { id, message }
    }
}

impl CommandTrait for ChatBroadcastCommand {
    fn execute(&self, zone: &mut zone::Zone) {
        log::info!(
            "Broadcasting chat message from {}: {}",
            self.id,
            self.message
        );
        let mut packet = proto::Packet::new();
        packet.set_category_text_message(proto::CategoryTextMessage::ChatReceive);
        let mut body = proto::PayloadTextMessage::new();
        body.set_id(self.id);
        body.set_message(self.message.to_string());
        let payload = body.serialize();
        if payload.is_err() {
            return;
        }
        packet.set_payload(payload.unwrap());
        zone.players_mut().iter_mut().for_each(|(_, cluster)| {
            cluster.stack_packet(packet.clone());
        });
    }
}
