use tokio::sync::oneshot;

use crate::{generated::proto_client::PayloadZoneExitNotification, zone::command::CommandTrait};

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

        let clients = zone.gateway_clients().clients_vec();

        zone.gateway_clients_mut().on_exit_player(&cluster);

        let message = PayloadZoneExitNotification { id: self.player_id };

        let _ = self.tx.send(true);
        tokio::spawn(async move {
            for mut client in clients {
                let _ = client.player_exit(message.clone()).await;
            }
        });
    }
}
