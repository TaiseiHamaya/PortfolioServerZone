use nalgebra::Point3;
use tokio::sync::mpsc;

use crate::zone::command::{CommandTrait, sync_service::event::*};

use crate::generated::proto_server::{
    PayloadPlayAction, PayloadTextMessage, PayloadTransformSync,
    zone_sync_service_server::ZoneSyncService,
};

pub struct ZoneSyncServiceImpl {
    pub command_sender: mpsc::Sender<Box<dyn CommandTrait + Send>>,
}

impl ZoneSyncServiceImpl {
    pub fn new(command_sender: mpsc::Sender<Box<dyn CommandTrait + Send>>) -> Self {
        Self { command_sender }
    }
}

#[tonic::async_trait]
impl ZoneSyncService for ZoneSyncServiceImpl {
    /// Zone → Client: player transform sync, play action, and chat message.
    async fn sync_transform(
        &self,
        request: tonic::Request<PayloadTransformSync>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let inner = request.into_inner();

        let Some(position_inner) = inner.position else {
            log::error!(
                "Position data missing in transform sync request for entity ID {}.",
                inner.id
            );
            return Err(tonic::Status::invalid_argument("Position data is required"));
        };

        let command = Box::new(sync_transform_command::SyncTransformCommand::new(
            inner.id,
            Point3::new(position_inner.x, position_inner.y, position_inner.z),
            inner.timestamp,
        ));

        self.command_sender.send(command).await.map_err(|_| {
            log::error!("Failed to send command to queue.");
            tonic::Status::internal("Failed to send command")
        })?;

        Ok(tonic::Response::new(()))
    }

    /// Zone → Client: player play action and chat message.
    async fn play_action(
        &self,
        request: tonic::Request<PayloadPlayAction>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let inner = request.into_inner();

        let command = Box::new(start_action_command::StartActionCommand::new(
            inner.id,
            inner.target_id,
            inner.action_id,
            inner.timestamp,
        ));

        self.command_sender.send(command).await.map_err(|_| {
            log::error!("Failed to send command to queue.");
            tonic::Status::internal("Failed to send command")
        })?;

        Ok(tonic::Response::new(()))
    }

    /// Zone → Client: player chat message.
    async fn send_chat(
        &self,
        request: tonic::Request<PayloadTextMessage>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let inner = request.into_inner();

        let command = Box::new(chat_broadcast_command::ChatBroadcastCommand::new(
            inner.id,
            inner.message,
        ));

        self.command_sender.send(command).await.map_err(|_| {
            log::error!("Failed to send command to queue.");
            tonic::Status::internal("Failed to send command")
        })?;

        Ok(tonic::Response::new(()))
    }
}
