use crate::zone::command::CommandTrait;

use crate::{generated::proto_client::PayloadZoneExitNotification, zone::zone};

pub struct DisconnectForceCommand {
    player_id: u64,
}

impl DisconnectForceCommand {
    pub fn new(player_id: u64) -> Self {
        DisconnectForceCommand { player_id }
    }
}

impl CommandTrait for DisconnectForceCommand {
    fn execute(self: Box<Self>, zone: &mut zone::Zone) {
        log::info!("Forcefully disconnecting player {}.", self.player_id);
        // ログアウト要求をチャッシュに追加
        zone.zone_request_chash_mut().push_logout(self.player_id);

        let mut client = zone
            .tonic_client_mut()
            .zone_broadcast_service_client
            .clone();

        let message = PayloadZoneExitNotification { id: self.player_id };

        tokio::spawn(async move {
            let result = client.player_exit(message).await;
            if let Err(e) = result {
                log::error!("Failed to notify player exit: {}", e);
            }
        });
    }
}
