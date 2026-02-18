use super::command_trait::CommandTrait;

use crate::{net::proto, zone::zone};
use protobuf::*;
pub struct DisconnectForceCommand {
    player_id: u64,
}

impl DisconnectForceCommand {
    pub fn new(player_id: u64) -> Self {
        DisconnectForceCommand { player_id }
    }
}

impl CommandTrait for DisconnectForceCommand {
    fn execute(&self, zone: &mut zone::Zone) {
        log::info!("Forcefully disconnecting player {}.", self.player_id);
        // ログアウト要求をチャッシュに追加
        zone.zone_request_chash_mut().push_logout(self.player_id);

        // 既存プレイヤーに通知するパケットを作成
        {
            let mut packet = proto::Packet::new();
            packet.set_category_logout_message(proto::CategoryLogoutMessage::LogoutNotification);
            let mut body = proto::PayloadLogoutNotification::new();
            body.set_id(self.player_id);
            let payload = body.serialize();
            if payload.is_ok() {
                packet.set_payload(payload.unwrap());
                zone.players_mut().iter_mut().for_each(|(_, cluster)| {
                    cluster.stack_packet(packet.clone());
                });
            }
        }
    }
}
