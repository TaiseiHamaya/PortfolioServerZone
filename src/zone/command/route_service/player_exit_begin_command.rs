use crate::{
    game::entity::entity::Entity,
    generated::proto_client::{self, PayloadPlayerRecord, PayloadPlayerSaveRequest},
    zone::command::{
        CommandBox, CommandTrait,
        route_service::{
            player_enter_execute_command, player_exit_wait_command, player_route_failed_fallback,
        },
    },
};

pub struct PlayerExitBeginCommand {
    user_id: u64,
}

impl PlayerExitBeginCommand {
    pub fn new(user_id: u64) -> Self {
        PlayerExitBeginCommand { user_id }
    }
}

impl CommandTrait for PlayerExitBeginCommand {
    fn execute(self: Box<Self>, zone: &mut crate::zone::zone::Zone) {
        log::info!("Player {} is beginning to exit the zone.", self.user_id);

        let mut db = zone
            .tonic_client_mut()
            .record_player_db_service_client
            .clone();

        let user_id = self.user_id;
        let player_cluster = match zone.players_mut().remove(&user_id) {
            Some(cluster) => cluster,
            None => {
                log::error!(
                    "Player with id {} is not in players when executing PlayerExitBeginCommand",
                    user_id
                );
                return;
            }
        };
        let username = player_cluster.player_name().clone();
        let position = player_cluster.player().position().clone();
        let current_zone_id = zone.id();

        // セーブしてコマンド化するタスク
        let task = async move {
            match db // DBから読み取り
                .save_player(PayloadPlayerSaveRequest {
                    record: Some(PayloadPlayerRecord {
                        user_id,
                        username,
                        position: Some(proto_client::Vector3 {
                            x: position.x,
                            y: position.y,
                            z: position.z,
                        }),
                        zone_id: current_zone_id,
                    }),
                })
                .await
            {
                Ok(response) => {
                    if response.into_inner().is_succeeded {
                        log::info!("Player data for user_id {} saved successfully", user_id);
                        return Some(Box::new(player_exit_wait_command::PlayerExitWait::new(
                            user_id,
                            player_cluster,
                        )) as CommandBox);
                    } else {
                        log::error!(
                            "Failed to save player data for user_id {}: Save operation failed",
                            user_id
                        );

                        // 失敗した場合、再入場させる
                        return Some(Box::new(
                            player_route_failed_fallback::PlayerRouteFailedFallback::new(
                                user_id,
                                Some(player_cluster),
                            ),
                        ) as CommandBox);
                    }
                }
                Err(e) => {
                    log::error!(
                        "Failed to load player data for user_id {}: {:?}",
                        user_id,
                        e
                    );
                    // 失敗した場合、再入場させる
                    return Some(Box::new(
                        player_route_failed_fallback::PlayerRouteFailedFallback::new(
                            user_id,
                            Some(player_cluster),
                        ),
                    ) as CommandBox);
                }
            }
        };

        zone.add_async_command(task);
    }
}
