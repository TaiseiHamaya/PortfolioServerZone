use crate::generated::proto_client::{
    record_player_db_service_client::RecordPlayerDbServiceClient,
    world_command_client::WorldCommandClient,
    zone_broadcast_service_client::ZoneBroadcastServiceClient,
};

#[derive(Clone)]
pub struct BackendClient {
    pub zone_broadcast_service_client: ZoneBroadcastServiceClient<tonic::transport::Channel>,
    pub world_command_client: WorldCommandClient<tonic::transport::Channel>,
    pub record_player_db_service_client: RecordPlayerDbServiceClient<tonic::transport::Channel>,
}

impl BackendClient {
    pub async fn new() -> Self {
        BackendClient {
            zone_broadcast_service_client: ZoneBroadcastServiceClient::connect(
                "http://[::1]:50051",
            )
            .await
            .expect("Failed to connect to Zone Broadcast Service"),
            world_command_client: WorldCommandClient::connect("http://[::1]:50052")
                .await
                .expect("Failed to connect to World Command Service"),
            record_player_db_service_client: RecordPlayerDbServiceClient::connect(
                "http://[::1]:50053",
            )
            .await
            .expect("Failed to connect to Record Player DB Service"),
        }
    }
}
