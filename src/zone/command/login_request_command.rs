use super::command_trait::CommandTrait;

use crate::{game::entity::entity::Entity, net::proto, zone::zone};
use protobuf::*;

pub struct LoginRequestCommand {
    player_id: u64,
    username: String,
}

impl LoginRequestCommand {
    pub fn new(player_id: u64, username: String) -> Self {
        LoginRequestCommand {
            player_id,
            username,
        }
    }
}

impl CommandTrait for LoginRequestCommand {
    fn execute(&self, zone: &mut zone::Zone) {
        log::info!(
            "Player {} ({}) requested login.",
            self.player_id,
            self.username
        );

        let players = zone.players_mut();

        // プレイヤー情報
        let mut player = players.get_mut(&self.player_id);
        if player.is_none() {
            return;
        }

        let player = player.as_mut().unwrap();

        // 他のプレイヤーに通知
        let mut packet = proto::Packet::new();
        packet.set_category_login_message(proto::CategoryLoginMessage::LoginNotification);
        let mut body = proto::PayloadLoginNotification::new();
        body.set_id(self.player_id);
        body.set_username(self.username.clone());
        let mut position = proto::Vector3::new();
        let player_position = player.player().position();
        position.set_x(player_position.x);
        position.set_y(player_position.y);
        position.set_z(player_position.z);
        body.set_position(position);

        packet.set_payload(body.serialize().unwrap());

        players.iter_mut().for_each(|(id, cluster)| {
            if *id == self.player_id {
                return; // 自分には送らない
            }
            cluster.stack_packet(packet.clone());
        });
    }
}
