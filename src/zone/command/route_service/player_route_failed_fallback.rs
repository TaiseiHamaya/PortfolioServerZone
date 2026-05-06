use crate::{net::client::Cluster, zone::command::CommandTrait};

pub struct PlayerRouteFailedFallback {
    user_id: u64,
    cluster: Option<Cluster>,
}

impl PlayerRouteFailedFallback {
    pub fn new(user_id: u64, cluster: Option<Cluster>) -> Self {
        PlayerRouteFailedFallback { user_id, cluster }
    }
}

impl CommandTrait for PlayerRouteFailedFallback {
    fn execute(self: Box<Self>, zone: &mut crate::zone::zone::Zone) {
        log::warn!(
            "Player {}({}) failed to route to the new zone. Executing fallback.",
            self.user_id,
            self.cluster.as_ref().map_or("Unknown", |c| c.player_name())
        );

        zone.routeing_players_mut().remove(&self.user_id);

        if let Some(cluster) = self.cluster {
            log::info!(
                "Re-adding player {}({}) to the zone after route failure.",
                cluster.entity_id(),
                cluster.player_name()
            );
            zone.players_mut().insert(self.user_id, cluster);
        }
    }
}
