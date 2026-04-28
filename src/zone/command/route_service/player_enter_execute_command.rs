use tokio::sync::oneshot;

use crate::{
    game::entity::entity::Entity, generated::proto_client::PayloadZoneEnterNotification,
    zone::command::CommandTrait,
};

pub struct PlayerEnterExecuteCommand {
    user_id: u64,
    tx: oneshot::Sender<Option<u64>>,
}

impl PlayerEnterExecuteCommand {
    pub fn new(player_id: u64, tx: oneshot::Sender<Option<u64>>) -> Self {
        PlayerEnterExecuteCommand {
            user_id: player_id,
            tx,
        }
    }
}

impl CommandTrait for PlayerEnterExecuteCommand {
    fn execute(self: Box<Self>, zone: &mut crate::zone::zone::Zone) {
        log::info!(
            "Executing PlayerEnterExecuteCommand for player_id: {}",
            self.user_id
        );

        let player_cluster = match zone.routeing_players_mut().remove(&self.user_id) {
            Some(entry) => entry,
            None => {
                log::error!(
                    "Player with id {} is not in routeing players when executing PlayerEnterExecuteCommand",
                    self.user_id
                );
                let _ = self.tx.send(None);
                return;
            }
        };

        let username = player_cluster.player_name().clone();
        let position = player_cluster.player().position().clone();

        zone.gateway_clients_mut().on_enter_player(&player_cluster);

        let entity_id = zone.next_entity_id();
        zone.players_mut().insert(entity_id, player_cluster);
        zone.player_id_by_user_id_mut()
            .insert(self.user_id, entity_id);

        let clients = zone.gateway_clients().clients_vec();
        let message = PayloadZoneEnterNotification {
            id: entity_id,
            username: username,
            position: Some(crate::generated::proto_client::Vector3 {
                x: position.x,
                y: position.y,
                z: position.z,
            }),
        };

        tokio::spawn(async move {
            for mut client in clients {
                let _ = client.player_enter(message.clone()).await;
            }
        });
        let _ = self.tx.send(Some(entity_id));
    }
}
