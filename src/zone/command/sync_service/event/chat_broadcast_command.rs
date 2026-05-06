use crate::zone::command::CommandTrait;
use crate::{
    generated::proto_server::{BroadcastStream, PayloadTextMessage, broadcast_stream},
    zone::zone,
};

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
    fn execute(self: Box<Self>, zone: &mut zone::Zone) {
        log::info!(
            "Broadcasting chat message from {}: {}",
            self.id,
            self.message
        );

        let message = PayloadTextMessage {
            id: self.id,
            message: self.message.clone(),
        };

        let message = BroadcastStream {
            payload: Some(broadcast_stream::Payload::ChatMessage(message)),
        };
        zone.send_broadcast_message(message);
    }
}
