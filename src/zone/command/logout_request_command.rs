use super::command_trait::CommandTrait;

use crate::{net::proto, zone::zone};
use protobuf::*;

pub struct LogoutRequestCommand {
    player_id: u64,
}

impl LogoutRequestCommand {
    pub fn new(player_id: u64) -> Self {
        LogoutRequestCommand { player_id }
    }
}

impl CommandTrait for LogoutRequestCommand {
    fn execute(&self, zone: &mut zone::Zone) {
        log::info!("Player {} requested disconnection.", self.player_id);
        // ログアウト要求をチャッシュに追加
        zone.zone_request_chash_mut().push_logout(self.player_id);
        // パケット送信
        // 要求を受けたクライアントに切断パケットを送信
        {
            if let Some(cluster) = zone.players_mut().get_mut(&self.player_id) {
                let mut packet = proto::Packet::new();
                packet.set_category_logout_message(proto::CategoryLogoutMessage::LogoutResponse);
                let mut body = proto::PayloadLogoutResponse::new();
                body.set_is_succeeded(true);
                let payload = body.serialize();
                if payload.is_ok() {
                    packet.set_payload(payload.unwrap());
                    cluster.stack_packet(packet);
                }
            }
        }

        // その他のクライアントにログアウト通知パケットを送信
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
