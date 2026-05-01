use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::mpsc;
use tonic::transport::Server;

use crate::{
    generated::proto_server::{
        BroadcastStream, world_route_service_server::WorldRouteServiceServer,
        zone_broadcast_service_server::ZoneBroadcastServiceServer,
        zone_sync_service_server::ZoneSyncServiceServer,
    },
    zone::command::CommandBox,
};

use super::{
    broadcast_event::ZoneBroadcastServiceImpl, world_service::WorldRouteServiceImpl,
    zone_sync::ZoneSyncServiceImpl,
};

pub struct BackendServerChannels {
    pub world_route_receiver: mpsc::Receiver<CommandBox>,
    pub sync_command_receiver: mpsc::Receiver<CommandBox>,

    pub broadcast_senders: Arc<DashMap<u64, mpsc::Sender<Result<BroadcastStream, tonic::Status>>>>,
}

pub async fn create_grpc_service(channel_size: usize, port: u16) -> BackendServerChannels {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);

    let (world_route_sender, world_command_receiver) = mpsc::channel::<CommandBox>(channel_size);
    let (sync_command_sender, sync_command_receiver) = mpsc::channel::<CommandBox>(channel_size);
    let senders = Arc::new(DashMap::new());

    tokio::spawn(
        Server::builder()
            .add_service(ZoneBroadcastServiceServer::new(
                ZoneBroadcastServiceImpl::new(senders.clone()),
            ))
            .add_service(WorldRouteServiceServer::new(WorldRouteServiceImpl::new(
                world_route_sender,
            )))
            .add_service(ZoneSyncServiceServer::new(ZoneSyncServiceImpl::new(
                sync_command_sender,
            )))
            .serve(addr),
    );

    BackendServerChannels {
        world_route_receiver: world_command_receiver,
        sync_command_receiver,
        broadcast_senders: senders,
    }
}
