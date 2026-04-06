use nalgebra::Point3;

use crate::{
    game::entity::player::Player,
    generated::proto_client::PayloadPlayerLoadRequest,
    net::client::Cluster,
    zone::command::{CommandBox, CommandTrait},
};

pub struct PlayerEnterBeginCommand {
    user_id: u64,
}

impl PlayerEnterBeginCommand {
    pub fn new(user_id: u64) -> Self {
        PlayerEnterBeginCommand { user_id }
    }
}

impl CommandTrait for PlayerEnterBeginCommand {
    fn execute(self: Box<Self>, zone: &mut crate::zone::zone::Zone) {
        log::info!("Player {} is beginning to enter the zone.", self.user_id);

        let mut db = zone
            .tonic_client_mut()
            .record_player_db_service_client
            .clone();

        let user_id = self.user_id;

        // ロードしてコマンド化するタスク
        let task = async move {
            match db // DBから読み取り
                .load_player(PayloadPlayerLoadRequest { user_id: user_id })
                .await
            {
                Ok(response) => {
                    let Some(player_record) = response.into_inner().record else {
                        // プレイヤーデータが見つかったがデータがおかしい
                        log::error!(
                            "Failed to load player data for user_id {}: No record found",
                            user_id
                        );
                        return None;
                    };

                    let Some(position) = player_record.position else {
                        // あるはずのデータがない
                        log::error!(
                            "Failed to load player data for user_id {}: No position data",
                            user_id
                        );
                        return None;
                    };

                    let player = Cluster::new(
                        Player::new(
                            0,
                            user_id,
                            Point3::new(position.x, position.y, position.z),
                            10000,
                        ),
                        player_record.username,
                    );

                    return Some(Box::new(
                        super::player_enter_execute_command::PlayerEnterExecuteCommand::new(
                            user_id, player,
                        ),
                    ) as CommandBox);
                }
                Err(e) => {
                    log::error!(
                        "Failed to load player data for user_id {}: {:?}",
                        user_id,
                        e
                    );
                    return None;
                }
            }
        };

        zone.add_async_command(task);
    }
}
