use tokio::sync::{mpsc, oneshot};

use crate::generated::proto_server::{
    PayloadPlayerZoneEnterReadyResponse, RoutePlayerData, Vector3,
};
use crate::zone::command::{CommandBox, route_service::*};

use crate::generated::proto_server::{
    PayloadPlayerExitZoneBegin, PayloadPlayerExitZoneComplete, PayloadPlayerExitZoneReady,
    PayloadPlayerRouteResult, PayloadPlayerZoneEnterBegin, PayloadPlayerZoneEnterComplete,
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
    ) -> std::result::Result<tonic::Response<PayloadPlayerZoneEnterReadyResponse>, tonic::Status>
    {
        let payload = request.into_inner();
        let user_id = payload.user_id;
        let gateway_id = payload.gateway_id;

        let (tx, rx) = oneshot::channel();

        match self
            .command_sender
            .send(Box::new(
                player_enter_begin_command::PlayerEnterBeginCommand::new(user_id, gateway_id, tx),
            ))
            .await
        {
            Ok(()) => match rx.await {
                Ok(Some((entity_id, position))) => {
                    return Ok(tonic::Response::new(PayloadPlayerZoneEnterReadyResponse {
                        player_data: Some(RoutePlayerData {
                            player_entity_id: entity_id,
                            position: Some(Vector3 {
                                x: position.x,
                                y: position.y,
                                z: position.z,
                            }),
                        }),
                    }));
                }
                Ok(None) => {
                    return Ok(tonic::Response::new(PayloadPlayerZoneEnterReadyResponse {
                        player_data: None,
                    }));
                }
                Err(e) => {
                    log::error!("Failed to receive PlayerEnterExecuteCommand result: {}", e);
                    return Err(tonic::Status::internal("Failed to receive command result"));
                }
            },
            Err(e) => {
                log::error!("Failed to send PlayerEnterExecuteCommand: {}", e);
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
        request: tonic::Request<PayloadPlayerZoneEnterComplete>,
    ) -> std::result::Result<tonic::Response<PayloadPlayerRouteResult>, tonic::Status> {
        let payload = request.into_inner();
        let user_id = payload.user_id;
        log::info!(
            "Received execute_enter_zone_player for user_id: {}",
            user_id
        );

        let (tx, rx) = oneshot::channel();

        match self
            .command_sender
            .send(Box::new(
                player_enter_execute_command::PlayerEnterExecuteCommand::new(user_id, tx),
            ))
            .await
        {
            Ok(()) => match rx.await {
                Ok(player_route_result) => {
                    log::info!(
                        "PlayerEnterExecuteCommand result for user_id {}: {:?}",
                        user_id,
                        player_route_result
                    );
                    return Ok(tonic::Response::new(PayloadPlayerRouteResult {
                        result: player_route_result as i32,
                    }));
                }
                Err(e) => {
                    log::error!("Failed to receive PlayerEnterExecuteCommand result: {}", e);
                    return Err(tonic::Status::internal("Failed to receive command result"));
                }
            },
            Err(e) => {
                log::error!("Failed to send PlayerEnterExecuteCommand: {}", e);
                return Err(tonic::Status::internal("Failed to send command"));
            }
        };
    }

    /// World → Zone: player exit zone request.
    async fn begin_player_zone_exit(
        &self,
        request: tonic::Request<PayloadPlayerExitZoneBegin>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let payload = request.into_inner();
        let user_id = payload.user_id;
        match self
            .command_sender
            .send(Box::new(
                player_exit_begin_command::PlayerExitBeginCommand::new(user_id),
            ))
            .await
        {
            Ok(()) => return Ok(tonic::Response::new(())),
            Err(e) => {
                log::error!("Failed to send PlayerExitBeginCommand: {}", e);
                return Err(tonic::Status::internal("Failed to send command"));
            }
        };
    }

    /// World → Zone: check if player is ready to exit the zone.
    async fn check_for_ready_exit_player(
        &self,
        request: tonic::Request<PayloadPlayerExitZoneReady>,
    ) -> std::result::Result<tonic::Response<PayloadPlayerRouteResult>, tonic::Status> {
        let payload = request.into_inner();
        let user_id = payload.user_id;

        let (tx, rx) = oneshot::channel();

        match self
            .command_sender
            .send(Box::new(
                player_exit_ready_command::PlayerExitReadyCommand::new(user_id, tx),
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
                    log::error!("Failed to receive PlayerExitReadyCommand result: {}", e);
                    return Err(tonic::Status::internal("Failed to receive command result"));
                }
            },
            Err(e) => {
                log::error!("Failed to send PlayerExitReadyCommand: {}", e);
                return Err(tonic::Status::internal("Failed to send command"));
            }
        };
    }

    /// World → Zone: execute player exit zone.
    async fn execute_exit_zone_player(
        &self,
        request: tonic::Request<PayloadPlayerExitZoneComplete>,
    ) -> std::result::Result<tonic::Response<PayloadPlayerRouteResult>, tonic::Status> {
        let payload = request.into_inner();
        let user_id = payload.user_id;
        log::info!("Received execute_exit_zone_player for user_id: {}", user_id);

        let (tx, rx) = oneshot::channel();

        match self
            .command_sender
            .send(Box::new(
                player_exit_execute_command::PlayerExitExecuteCommand::new(user_id, tx),
            ))
            .await
        {
            Ok(()) => match rx.await {
                Ok(is_success) => {
                    if is_success {
                        return Ok(tonic::Response::new(PayloadPlayerRouteResult {
                            result: PlayerRouteResult::Success as i32,
                        }));
                    } else {
                        return Ok(tonic::Response::new(PayloadPlayerRouteResult {
                            result: PlayerRouteResult::Failed as i32,
                        }));
                    }
                }
                Err(e) => {
                    log::error!("Failed to receive PlayerExitExecuteCommand result: {}", e);
                    return Err(tonic::Status::internal("Failed to receive command result"));
                }
            },
            Err(e) => {
                log::error!("Failed to send PlayerExitExecuteCommand: {}", e);
                return Err(tonic::Status::internal("Failed to send command"));
            }
        };
    }
}
