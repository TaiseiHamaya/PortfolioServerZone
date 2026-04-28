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

        tokio::spawn(async move {
            for mut client in clients {
                let result = client.send_chat(message.clone()).await;
                if let Err(e) = result {
                    log::error!("Failed to broadcast chat message: {}", e);
                }
            }
        });
    }
}
