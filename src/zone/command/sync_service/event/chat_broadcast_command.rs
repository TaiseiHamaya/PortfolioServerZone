use crate::zone::command::CommandTrait;

use crate::{generated::proto_client::PayloadTextMessage, zone::zone};

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

        let mut client = zone
            .tonic_client_mut()
            .zone_broadcast_service_client
            .clone();

        let message = PayloadTextMessage {
            id: self.id,
            message: self.message.clone(),
        };

        tokio::spawn(async move {
            let result = client.send_chat(message).await;
            if let Err(e) = result {
                log::error!("Failed to broadcast chat message: {}", e);
            }
        });
    }
}
