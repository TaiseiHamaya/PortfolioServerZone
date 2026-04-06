use tokio::sync::{mpsc, oneshot};

use crate::zone::command::{CommandBox, route_service::*};

use crate::generated::proto_server::{
    PayloadPlayerEnterZoneComplete, PayloadPlayerExitZoneBegin, PayloadPlayerExitZoneComplete,
    PayloadPlayerExitZoneReady, PayloadPlayerRouteResult, PayloadPlayerZoneEnterBegin,
    PayloadPlayerZoneEnterReady, PlayerRouteResult, world_route_service_server::WorldRouteService,
};

pub struct WorldRouteServiceImpl {
    pub command_sender: mpsc::Sender<CommandBox>,
}

impl WorldRouteServiceImpl {
    pub fn new(command_sender: mpsc::Sender<CommandBox>) -> Self {
        Self { command_sender }
    }
}

#[tonic::async_trait]
impl WorldRouteService for WorldRouteServiceImpl {
    /// World → Zone: player route request.
    async fn begin_player_zone_enter(
        &self,
        request: tonic::Request<PayloadPlayerZoneEnterBegin>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let payload = request.into_inner();
        let user_id = payload.user_id;
        match self
            .command_sender
            .send(Box::new(
                player_enter_begin_command::PlayerEnterBeginCommand::new(user_id),
            ))
            .await
        {
            Ok(()) => return Ok(tonic::Response::new(())),
            Err(e) => {
                log::error!("Failed to send PlayerEnterBeginCommand: {}", e);
                return Err(tonic::Status::internal("Failed to send command"));
            }
        };
    }

    /// World → Zone: check if player is ready to enter the zone.
    async fn check_for_ready_enter_player(
        &self,
        request: tonic::Request<PayloadPlayerZoneEnterReady>,
    ) -> std::result::Result<tonic::Response<PayloadPlayerRouteResult>, tonic::Status> {
        let payload = request.into_inner();
        let user_id = payload.user_id;

        let (tx, rx) = oneshot::channel();

        match self
            .command_sender
            .send(Box::new(
                player_enter_ready_command::PlayerEnterReadyCommand::new(user_id, tx),
            ))
            .await
        {
            Ok(()) => match rx.await {
                Ok(is_ready) => {
                    if is_ready {
                        return Ok(tonic::Response::new(PayloadPlayerRouteResult {
                            result: PlayerRouteResult::Success as i32,
                        }));
                    } else {
                        return Ok(tonic::Response::new(PayloadPlayerRouteResult {
                            result: PlayerRouteResult::NotReady as i32,
                        }));
                    }
                }
                Err(e) => {
                    log::error!("Failed to receive PlayerEnterReadyCommand result: {}", e);
                    return Err(tonic::Status::internal("Failed to receive command result"));
                }
            },
            Err(e) => {
                log::error!("Failed to send PlayerEnterReadyCommand: {}", e);
                return Err(tonic::Status::internal("Failed to send command"));
            }
        };
    }

    /// World → Zone: execute player enter zone.
    async fn execute_enter_zone_player(
        &self,
        request: tonic::Request<PayloadPlayerEnterZoneComplete>,
    ) -> std::result::Result<tonic::Response<PayloadPlayerRouteResult>, tonic::Status> {
        todo!()
    }

    /// World → Zone: player exit zone request.
    async fn begin_player_zone_exit(
        &self,
        request: tonic::Request<PayloadPlayerExitZoneBegin>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        todo!()
    }

    /// World → Zone: check if player is ready to exit the zone.
    async fn check_for_ready_exit_player(
        &self,
        request: tonic::Request<PayloadPlayerExitZoneReady>,
    ) -> std::result::Result<tonic::Response<PayloadPlayerRouteResult>, tonic::Status> {
        todo!()
    }

    /// World → Zone: execute player exit zone.
    async fn execute_exit_zone_player(
        &self,
        request: tonic::Request<PayloadPlayerExitZoneComplete>,
    ) -> std::result::Result<tonic::Response<PayloadPlayerRouteResult>, tonic::Status> {
        todo!()
    }
}
