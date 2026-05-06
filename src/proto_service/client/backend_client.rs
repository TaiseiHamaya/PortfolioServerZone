use tonic::transport::Endpoint;

use crate::generated::proto_client::{
    record_player_db_service_client::RecordPlayerDbServiceClient,
    world_command_client::WorldCommandClient,
};

#[derive(Clone)]
pub struct BackendClient {
    #[allow(dead_code)]
    pub world_command_client: WorldCommandClient<tonic::transport::Channel>,
    pub record_player_db_service_client: RecordPlayerDbServiceClient<tonic::transport::Channel>,
}

impl BackendClient {
    pub async fn new(
        world_command_endpoint: Endpoint,
        record_player_db_endpoint: Endpoint,
    ) -> Self {
        let world_command_client = WorldCommandClient::connect(world_command_endpoint)
            .await
            .expect("Failed to connect to World Command Service");

        let record_player_db_service_client =
            RecordPlayerDbServiceClient::connect(record_player_db_endpoint)
                .await
                .expect("Failed to connect to Record Player DB Service");

        BackendClient {
            world_command_client,
            record_player_db_service_client,
        }
    }
}
