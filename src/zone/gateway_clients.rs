use std::sync::Arc;

use dashmap::DashMap;

use crate::{
    generated::proto_client::zone_broadcast_service_client::ZoneBroadcastServiceClient,
    net::client::Cluster,
};

pub struct GatewayClients {
    channels: Arc<DashMap<u64, tonic::transport::Channel>>,

    clients: DashMap<u64, (ZoneBroadcastServiceClient<tonic::transport::Channel>, i64)>,
}

impl GatewayClients {
    pub fn new(channels: Arc<DashMap<u64, tonic::transport::Channel>>) -> Self {
        GatewayClients {
            channels,
            clients: DashMap::new(),
        }
    }

    pub fn on_enter_player(&mut self, player: &Cluster) {
        let gateway_id = player.gateway_id();

        // get channel
        let Some(channel) = self.channels.get(&gateway_id) else {
            log::error!("Gateway channel for gateway_id {} not found", gateway_id);
            return;
        };

        // clone channel
        let channel = channel.value().clone();

        // get or insert client
        let mut client = self.clients.entry(gateway_id).or_insert_with(|| {
            log::info!("Creating new gateway client for gateway_id {}", gateway_id);
            let client = ZoneBroadcastServiceClient::new(channel);
            (client, 0)
        });

        // use count
        client.1 += 1;
        log::info!(
            "Player entered gateway_id {}. Current use count: {}",
            gateway_id,
            client.1
        );
    }

    pub fn on_exit_player(&mut self, player: &Cluster) {
        let gateway_id = player.gateway_id();

        let Some(mut client) = self.clients.get_mut(&gateway_id) else {
            log::error!(
                "Gateway client for gateway_id {} not found on player exit",
                gateway_id
            );
            return;
        };

        // decrease use count
        client.1 -= 1;

        log::info!(
            "Player exited gateway_id {}. Current use count: {}",
            gateway_id,
            client.1
        );
        // if no more players using this client, remove it
        if client.1 <= 0 {
            drop(client);
            self.clients.remove(&gateway_id);
            log::info!("Gateway client for gateway_id {} removed", gateway_id);
        }
    }

    pub fn clients_vec(&self) -> Vec<ZoneBroadcastServiceClient<tonic::transport::Channel>> {
        self.clients
            .iter()
            .map(|entry| entry.value().0.clone())
            .collect()
    }
}
