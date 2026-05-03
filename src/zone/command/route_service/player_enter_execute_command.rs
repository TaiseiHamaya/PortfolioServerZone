use tokio::sync::oneshot;

use crate::{
    game::entity::entity::Entity,
    generated::proto_server::{
        BroadcastStream, EnemyData, EntityData, PayloadClientInitializerData,
        PayloadZoneEnterNotification, PlayerData, PlayerRouteResult, Vector3, broadcast_stream,
    },
    zone::command::CommandTrait,
};

pub struct PlayerEnterExecuteCommand {
    user_id: u64,
    tx: oneshot::Sender<PlayerRouteResult>,
}

impl PlayerEnterExecuteCommand {
    pub fn new(player_id: u64, tx: oneshot::Sender<PlayerRouteResult>) -> Self {
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
                let _ = self.tx.send(PlayerRouteResult::Failed);
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

        let _ = self.tx.send(PlayerRouteResult::Success);

        // initial data
        let data = PayloadClientInitializerData {
            enemies: zone
                .contains_directors_mut()
                .iter()
                .flat_map(|director| director.enemies())
                .map(|(_, enemy)| EnemyData {
                    enemy_type_id: enemy.enemy_type_id(),
                    entity_data: Some(EntityData {
                        entity_id: enemy.entity_id(),
                        hp: enemy.hitpoint(),
                        position: Some(Vector3 {
                            x: enemy.position().x,
                            y: enemy.position().y,
                            z: enemy.position().z,
                        }),
                    }),
                })
                .collect(),
            players: zone
                .players_mut()
                .iter()
                .map(|(entity_id, cluster)| {
                    let player = cluster.player();
                    PlayerData {
                        entity_data: Some(EntityData {
                            entity_id: *entity_id,
                            hp: player.hitpoint(),
                            position: Some(Vector3 {
                                x: player.position().x,
                                y: player.position().y,
                                z: player.position().z,
                            }),
                        }),
                        name: cluster.player_name().clone(),
                    }
                })
                .collect(),
        };
        log::info!(
            "Sending ClientInitializerData to gateway_id: {}, user_id: {}, players_count: {}, enemies_count: {}",
            gateway_id,
            self.user_id,
            data.players.len(),
            data.enemies.len(),
        );
        let message = BroadcastStream {
            payload: Some(broadcast_stream::Payload::ClientInitializerData(data)),
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
