use std::sync::Arc;

use dashmap::{DashMap, Entry};
use tokio::sync::mpsc;
use tokio_stream;
use tonic;

use crate::generated::proto_server::{
    BeginConnectGateway, BroadcastStream, zone_broadcast_service_server::ZoneBroadcastService,
};

pub struct ZoneBroadcastServiceImpl {
    senders: Arc<DashMap<u64, mpsc::Sender<Result<BroadcastStream, tonic::Status>>>>,
}

impl ZoneBroadcastServiceImpl {
    pub fn new(
        senders: Arc<DashMap<u64, mpsc::Sender<Result<BroadcastStream, tonic::Status>>>>,
    ) -> Self {
        Self { senders: senders }
    }
}

#[tonic::async_trait]
impl ZoneBroadcastService for ZoneBroadcastServiceImpl {
    type ZoneSyncStreamStream =
        tokio_stream::wrappers::ReceiverStream<Result<BroadcastStream, tonic::Status>>;

    async fn zone_sync_stream(
        &self,
        request: tonic::Request<BeginConnectGateway>,
    ) -> std::result::Result<
        tonic::Response<<ZoneBroadcastServiceImpl as ZoneBroadcastService>::ZoneSyncStreamStream>,
        tonic::Status,
    > {
        let payload = request.into_inner();
        let gateway_id = payload.gateway_id;

        log::info!(
            "Gateway {} connected to Zone Broadcast Service.",
            gateway_id
        );

        match self.senders.entry(gateway_id) {
            Entry::Occupied(_) => {
                log::warn!(
                    "Gateway {} is already connected. Connection rejected.",
                    gateway_id
                );
                Err(tonic::Status::already_exists("Gateway already connected"))
            }
            Entry::Vacant(vacant) => {
                let (tx, rx) = tokio::sync::mpsc::channel(1024);
                vacant.insert(tx);
                log::info!(
                    "Gateway {} successfully connected to Zone Broadcast Service.",
                    gateway_id
                );

                Ok(tonic::Response::new(
                    tokio_stream::wrappers::ReceiverStream::new(rx),
                ))
            }
        }
    }
}
