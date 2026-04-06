use crate::zone::command::CommandTrait;

use crate::{
    game::entity::entity::Entity, generated::proto_client::PayloadZoneEnterNotification, zone::zone,
};

pub struct PlayerEnterCommand {
    player_id: u64,
    username: String,
}

impl PlayerEnterCommand {
    pub fn new(player_id: u64, username: String) -> Self {
        PlayerEnterCommand {
            player_id,
            username,
        }
    }
}

impl CommandTrait for PlayerEnterCommand {
    fn execute(self: Box<Self>, zone: &mut zone::Zone) {
        log::info!(
            "Player {} ({}) entered the zone.",
            self.player_id,
            self.username
        );

        let players = zone.players_mut();

        // プレイヤー情報
        let player = players.get(&self.player_id);
        if player.is_none() {
            return;
        }

        let player = player.unwrap();
        let position = player.player().position().clone();

        let mut client = zone
            .tonic_client_mut()
            .zone_broadcast_service_client
            .clone();

        let message = PayloadZoneEnterNotification {
            id: self.player_id,
            username: self.username.clone(),
            position: Some(crate::generated::proto_client::Vector3 {
                x: position.x,
                y: position.y,
                z: position.z,
            }),
        };

        tokio::spawn(async move {
            if let Err(e) = client.player_enter(message).await {
                log::error!("Failed to send player enter notification: {}", e);
            }
        });
    }
}
