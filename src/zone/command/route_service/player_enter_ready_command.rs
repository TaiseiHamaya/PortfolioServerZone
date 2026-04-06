use tokio::sync::oneshot;

use crate::zone::command::CommandTrait;

pub struct PlayerEnterReadyCommand {
    player_id: u64,
    tx: oneshot::Sender<bool>,
}

impl PlayerEnterReadyCommand {
    pub fn new(player_id: u64, tx: oneshot::Sender<bool>) -> Self {
        PlayerEnterReadyCommand { player_id, tx }
    }
}

impl CommandTrait for PlayerEnterReadyCommand {
    fn execute(self: Box<Self>, zone: &mut crate::zone::zone::Zone) {
        let is_ready = zone.players_mut().contains_key(&self.player_id);

        if let Err(e) = self.tx.send(is_ready) {
            log::error!("Failed to send player enter ready result: {}", e);
        }
    }
}
