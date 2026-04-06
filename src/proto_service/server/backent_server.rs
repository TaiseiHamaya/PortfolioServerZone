use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::sync::mpsc;
use tonic::transport::Server;

use crate::{
    generated::proto_server::{
        world_route_service_server::WorldRouteServiceServer,
        zone_sync_service_server::ZoneSyncServiceServer,
    },
    zone::command::CommandBox,
};

use super::{world_service::WorldRouteServiceImpl, zone_sync::ZoneSyncServiceImpl};

pub struct BackendServerReceiver {
    pub world_command_receiver: mpsc::Receiver<CommandBox>,
    pub sync_command_receiver: mpsc::Receiver<CommandBox>,
}

pub async fn create_backend_server(channel_size: usize) -> BackendServerReceiver {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 50051);

    let (world_command_sender, world_command_receiver) = mpsc::channel::<CommandBox>(channel_size);
    let (sync_command_sender, sync_command_receiver) = mpsc::channel::<CommandBox>(channel_size);

    Server::builder()
        .add_service(WorldRouteServiceServer::new(WorldRouteServiceImpl::new(
            world_command_sender,
        )))
        .add_service(ZoneSyncServiceServer::new(ZoneSyncServiceImpl::new(
            sync_command_sender,
        )))
        .serve(addr)
        .await
        .expect("Failed to serve");

    BackendServerReceiver {
        world_command_receiver,
        sync_command_receiver,
    }
}
