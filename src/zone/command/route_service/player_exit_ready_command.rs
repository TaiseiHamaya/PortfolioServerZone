use tokio::sync::oneshot;

use crate::zone::command::CommandTrait;

pub struct PlayerExitReadyCommand {
    player_id: u64,
    tx: oneshot::Sender<bool>,
}

impl PlayerExitReadyCommand {
    pub fn new(player_id: u64, tx: oneshot::Sender<bool>) -> Self {
        PlayerExitReadyCommand { player_id, tx }
    }
}

impl CommandTrait for PlayerExitReadyCommand {
    fn execute(self: Box<Self>, zone: &mut crate::zone::zone::Zone) {
        let is_ready = zone.routeing_players_mut().contains_key(&self.player_id);

        if let Err(e) = self.tx.send(is_ready) {
            log::error!("Failed to send player exit ready result: {}", e);
        }
    }
}
