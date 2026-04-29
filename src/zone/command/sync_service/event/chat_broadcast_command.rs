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

        let clients = zone.gateway_clients().clients_vec();

        let message = PayloadTextMessage {
            id: self.id,
            message: self.message.clone(),
        };

        for mut client in clients {
            let message_clone = message.clone();
            tokio::spawn(async move {
                if let Err(e) = client.send_chat(message_clone).await {
                    log::error!("Failed to broadcast chat message: {}", e);
                }
            });
        }
    }
}
