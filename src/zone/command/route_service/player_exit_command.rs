use crate::zone::command::CommandTrait;

use crate::{generated::proto_client::PayloadZoneExitNotification, zone::zone};

pub struct PlayerExitCommand {
    player_id: u64,
}

impl PlayerExitCommand {
    pub fn new(player_id: u64) -> Self {
        PlayerExitCommand { player_id }
    }
}

impl CommandTrait for PlayerExitCommand {
    fn execute(self: Box<Self>, zone: &mut zone::Zone) {
        log::info!("Player {} requested to exit the zone.", self.player_id);
        // プレイヤー退出要求をチャッシュに追加
        zone.zone_request_chash_mut().push_logout(self.player_id);

        let mut client = zone
            .tonic_client_mut()
            .zone_broadcast_service_client
            .clone();

        let message = PayloadZoneExitNotification { id: self.player_id };

        tokio::spawn(async move {
            if let Err(e) = client.player_exit(message).await {
                log::error!("Failed to send player exit notification: {}", e);
            }
        });
    }
}
