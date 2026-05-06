use crate::net::client::Cluster;
use crate::zone::command::CommandTrait;
use crate::zone::zone;

pub struct PlayerEnterWait {
    player_id: u64,
    player_data: Cluster,
}

impl PlayerEnterWait {
    pub fn new(player_id: u64, player_data: Cluster) -> Self {
        PlayerEnterWait {
            player_id,
            player_data,
        }
    }
}

impl CommandTrait for PlayerEnterWait {
    fn execute(self: Box<Self>, zone: &mut zone::Zone) {
        log::info!(
            "Player {} ({}) entered the zone.",
            self.player_id,
            self.player_data.player_name()
        );

        zone.routeing_players_mut()
            .insert(self.player_id, self.player_data);
    }
}
