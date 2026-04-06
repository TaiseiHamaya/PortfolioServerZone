use crate::{net::client::Cluster, zone::command::CommandTrait};

pub struct PlayerEnterExecuteCommand {
    player_id: u64,
    player: Cluster,
}

impl PlayerEnterExecuteCommand {
    pub fn new(player_id: u64, player: Cluster) -> Self {
        PlayerEnterExecuteCommand { player_id, player }
    }
}

impl CommandTrait for PlayerEnterExecuteCommand {
    fn execute(self: Box<Self>, zone: &mut crate::zone::zone::Zone) {
        log::info!("Player {} is executing the zone entry.", self.player_id);
    }
}
