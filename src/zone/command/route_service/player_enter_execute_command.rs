use nalgebra::Point3;
use tokio::sync::oneshot;

use crate::{
    game::entity::entity::Entity,
    generated::proto_server::{
        BroadcastStream, PayloadClientInitializerData, PayloadZoneEnterNotification, Vector3,
        broadcast_stream,
    },
    zone::command::CommandTrait,
};

pub struct PlayerEnterExecuteCommand {
    user_id: u64,
    tx: oneshot::Sender<Option<(u64, Point3<f32>)>>,
}

impl PlayerEnterExecuteCommand {
    pub fn new(player_id: u64, tx: oneshot::Sender<Option<(u64, Point3<f32>)>>) -> Self {
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

        let gateway_id = player_cluster.gateway_id();

        let username = player_cluster.player_name().clone();
        let position = player_cluster.player().position().clone();

        let entity_id = player_cluster.entity_id();
        zone.players_mut().insert(entity_id, player_cluster);
        zone.player_id_by_user_id_mut()
            .insert(self.user_id, entity_id);

        let _ = self.tx.send(Some((entity_id, position)));

        // initial data
        let message = BroadcastStream {
            payload: Some(broadcast_stream::Payload::ClientInitializerData(
                PayloadClientInitializerData {
                    enemies: vec![],
                    players: vec![],
                },
            )),
        };
        zone.send_broadcast_message_to_gateway(gateway_id, message);

        // notification
        let message = BroadcastStream {
            payload: Some(broadcast_stream::Payload::PlayerEnter(
                PayloadZoneEnterNotification {
                    id: entity_id,
                    username,
                    position: Some(Vector3 {
                        x: position.x,
                        y: position.y,
                        z: position.z,
                    }),
                },
            )),
        };
        zone.send_broadcast_message(message);
    }
}
