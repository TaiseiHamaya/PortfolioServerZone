use dashmap::{DashMap, mapref::entry};
use std::sync::RwLock;

use crate::generated::proto_client::{
    record_player_db_service_client::RecordPlayerDbServiceClient,
    world_command_client::WorldCommandClient,
    zone_broadcast_service_client::ZoneBroadcastServiceClient,
};

#[derive(Clone)]
pub struct BackendClient {
    pub zone_broadcast_service_clients:
        DashMap<u64, ZoneBroadcastServiceClient<tonic::transport::Channel>>,
    pub world_command_client: WorldCommandClient<tonic::transport::Channel>,
    pub record_player_db_service_client: RecordPlayerDbServiceClient<tonic::transport::Channel>,
}

impl BackendClient {
    pub async fn new() -> Self {
        let world_command_client = WorldCommandClient::connect("http://localhost:50051")
            .await
            .expect("Failed to connect to World Command Service");
        let record_player_db_service_client =
            RecordPlayerDbServiceClient::connect("http://localhost:50050")
                .await
                .expect("Failed to connect to Record Player DB Service");
        BackendClient {
            zone_broadcast_service_clients: DashMap::new(),
            world_command_client,
            record_player_db_service_client,
        }
    }

    pub async fn add_gateway_client(
        &self,
        gateway_id: u64,
        client: ZoneBroadcastServiceClient<tonic::transport::Channel>,
    ) {
        self.zone_broadcast_service_clients
            .entry(gateway_id)
            .or_insert(client);
    }

    pub async fn remove_gateway_client(&self, gateway_id: u64) {
        self.zone_broadcast_service_clients.remove(&gateway_id);
    }

    pub fn get_gateway_clients(
        &self,
    ) -> Vec<ZoneBroadcastServiceClient<tonic::transport::Channel>> {
        self.zone_broadcast_service_clients
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
}
