use chrono::TimeZone;
use chrono::offset::LocalResult;
use log;
use nalgebra::Point3;
use protobuf::ClearAndParse;
use protobuf::Serialize;

use super::tcp_client::*;

use crate::zone::command;
use crate::{
    game::entity::{
        entity::{Entity, PlayActionError, PlayActionOk},
        player::Player,
    },
    net::proto,
};

pub struct Cluster<'action_list> {
    player: Player<'action_list>,
    name: String,
    tcp: TcpClient,

    command_buffers: Vec<Box<dyn command::CommandTrait>>,
}

impl<'action_list> Cluster<'action_list> {
    pub fn new(player: Player<'action_list>, name: String, tcp: TcpClient) -> Self {
        Cluster {
            player,
            name,
            tcp,
            command_buffers: Vec::new(),
        }
    }

    pub async fn recv_packets(&mut self) {
        self.tcp.recv().await;
    }

    pub fn process_packets(&mut self) {
        let messages = self.tcp.into_recv_messages();
        for msg in messages {
            match msg.category() {
                proto::packet::CategoryOneof::CategoryLoginMessage(
                    proto::CategoryLoginMessage::LoginRequest,
                ) => {
                    // ログインリクエスト
                    let mut login_packet = proto::PayloadLoginRequest::new();
                    let parsed = login_packet.clear_and_parse(&msg.payload());
                    if parsed.is_err() {
                        log::error!("Failed to parse LoginRequestBody: {:?}", parsed.err());
                        continue;
                    }
                    let username = login_packet.username().to_string();
                    log::info!("Player {} is attempting to log in.", username);
                    self.name = username;
                    self.command_buffers
                        .push(Box::new(command::LoginRequestCommand::new(
                            self.player.id(),
                            self.name.clone(),
                        )));
                }
                proto::packet::CategoryOneof::CategoryLogoutMessage(
                    proto::CategoryLogoutMessage::LogoutRequest,
                ) => {
                    // ログアウトリクエスト
                    self.command_buffers
                        .push(Box::new(command::LogoutRequestCommand::new(
                            self.player.id(),
                        )));
                }
                proto::packet::CategoryOneof::CategorySyncMessage(
                    proto::CategorySyncMessage::SyncTransform,
                ) => {
                    // 同期
                    let mut transform_packet = proto::PayloadTransformSync::new();
                    let parsed = transform_packet.clear_and_parse(&msg.payload());
                    if parsed.is_err() {
                        log::error!("Failed to parse TransformSyncBody: {:?}", parsed.err());
                        continue;
                    }
                    let position = transform_packet.position();

                    let player_position = self.player.position_mut();
                    *player_position = Point3::new(position.x(), position.y(), position.z());
                }
                proto::packet::CategoryOneof::CategorySyncMessage(
                    proto::CategorySyncMessage::PlayAction,
                ) => {
                    // アクション実行
                    let mut action_packet = proto::PayloadPlayAction::new();
                    let parsed = action_packet.clear_and_parse(&msg.payload());
                    if parsed.is_err() {
                        log::error!("Failed to parse PlayActionBody: {:?}", parsed.err());
                        continue;
                    }
                    let action_id = action_packet.action_id();
                    let target_id = action_packet.entity_id();
                    let timestamp = chrono::Utc.timestamp_micros(action_packet.timestamp() as i64);
                    match timestamp {
                        LocalResult::Single(time) => {
                            log::info!(
                                "Player {} performs action {} at {}",
                                self.player.id(),
                                action_id,
                                time
                            );
                            let action = self.player.play_action(action_id, &time);
                            if action.is_err() {
                                log::warn!(
                                    "Player {} failed to perform action {}: {:?}",
                                    self.player.id(),
                                    action_id,
                                    action.err().unwrap()
                                );
                                continue;
                            }
                            match action.unwrap() {
                                PlayActionOk::Damage(dmg) => {
                                    self.command_buffers.push(Box::new(
                                        command::StartActionCommand::new(
                                            self.player.id(),
                                            action_id,
                                        ),
                                    ));
                                    self.command_buffers.push(Box::new(
                                        command::DamagedEntityCommand::new(
                                            self.player.id(),
                                            target_id,
                                            dmg,
                                        ),
                                    ));
                                }
                            };
                        }
                        _ => {
                            log::warn!(
                                "Player {} sent invalid timestamp for action {}",
                                self.player.id(),
                                action_id
                            );
                        }
                    }
                }
                proto::packet::CategoryOneof::CategoryTextMessage(
                    proto::CategoryTextMessage::ChatSend,
                ) => {
                    // テキストチャット
                    let mut chat_packet = proto::PayloadTextMessage::new();
                    let parsed = chat_packet.clear_and_parse(&msg.payload());
                    if parsed.is_err() {
                        log::error!("Failed to parse ChatSendBody: {:?}", parsed.err());
                        continue;
                    }
                    let message = chat_packet.message().to_string();
                    log::info!("Player {} says: {}", self.player.id(), message);
                    self.command_buffers
                        .push(Box::new(command::ChatBroadcastCommand::new(
                            self.player.id(),
                            message,
                        )));
                }
                _ => {
                    // 不明なパケット
                    println!(
                        "Unknown packet type received from player {}: {:?}",
                        self.player.id(),
                        msg.category()
                    );
                }
            }
        }
    }

    pub fn update(&mut self) {
        self.player.update();

        if self.tcp.check_error() {
            self.command_buffers
                .push(Box::new(command::DisconnectForceCommand::new(
                    self.player.id(),
                )));
        }
        if self.tcp.is_disconnected() {
            self.command_buffers
                .push(Box::new(command::LogoutRequestCommand::new(
                    self.player.id(),
                )));
        }
    }

    pub fn send_packets(&mut self) {
        self.tcp.send();
    }

    pub fn take_commands(&mut self) -> Vec<Box<dyn command::CommandTrait>> {
        let commands = std::mem::take(&mut self.command_buffers);
        commands
    }

    pub fn stack_packet(&mut self, packet: proto::Packet) {
        self.tcp.stack_packet(packet);
    }

    pub fn on_accepted(&mut self) {
        let mut notify_packet = proto::Packet::new();
        notify_packet.set_category_login_message(proto::CategoryLoginMessage::LoginResult); // パケットタイプ
        let mut payload = proto::PayloadLoginResult::new();
        payload.set_id(self.player.id());
        payload.set_is_succeeded(true);
        payload.set_position(proto::Vector3::new());
        notify_packet.set_payload(payload.serialize().unwrap()); // 中身
        self.tcp.stack_packet(notify_packet);
    }

    pub fn id(&self) -> u64 {
        self.player.id()
    }

    pub fn player_name(&self) -> &String {
        &self.name
    }

    pub fn player(&self) -> &Player<'action_list> {
        &self.player
    }
}
