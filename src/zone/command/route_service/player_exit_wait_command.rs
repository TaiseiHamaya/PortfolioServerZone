use crate::zone::command::CommandTrait;

use crate::net::client::Cluster;
use crate::zone::zone;

pub struct PlayerExitWait {
    player_id: u64,
    player_cluster: Cluster,
}

impl PlayerExitWait {
    pub fn new(player_id: u64, player_cluster: Cluster) -> Self {
        PlayerExitWait {
            player_id,
            player_cluster,
        }
    }
}

impl CommandTrait for PlayerExitWait {
    fn execute(self: Box<Self>, zone: &mut zone::Zone) {
        log::info!(
            "Player {}({}) has finished exiting the zone and is now being removed.",
            self.player_id,
            self.player_cluster.player_name()
        );

        zone.routeing_players_mut()
            .insert(self.player_id, self.player_cluster);
    }
}
