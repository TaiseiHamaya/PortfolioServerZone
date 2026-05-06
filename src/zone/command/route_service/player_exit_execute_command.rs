use tokio::sync::oneshot;

use crate::{
    generated::proto_server::{BroadcastStream, PayloadZoneExitNotification, broadcast_stream},
    zone::command::CommandTrait,
};

pub struct PlayerExitExecuteCommand {
    player_id: u64,
    tx: oneshot::Sender<bool>,
}

impl PlayerExitExecuteCommand {
    pub fn new(player_id: u64, tx: oneshot::Sender<bool>) -> Self {
        PlayerExitExecuteCommand { player_id, tx }
    }
}

impl CommandTrait for PlayerExitExecuteCommand {
    fn execute(self: Box<Self>, zone: &mut crate::zone::zone::Zone) {
        log::info!(
            "Executing PlayerExitExecuteCommand for player_id: {}",
            self.player_id
        );

        let cluster = match zone.routeing_players_mut().remove(&self.player_id) {
            Some(cluster) => cluster,
            None => {
                log::error!(
                    "Player with id {} is not in routeing players when executing PlayerExitExecuteCommand",
                    self.player_id
                );
                let _ = self.tx.send(false);
                return;
            }
        };

        let entity_id = cluster.entity_id();

        let _ = self.tx.send(true);

        // notification
        let message = BroadcastStream {
            payload: Some(broadcast_stream::Payload::PlayerExit(
                PayloadZoneExitNotification { id: entity_id },
            )),
        };
        zone.send_broadcast_message(message);
    }
}
