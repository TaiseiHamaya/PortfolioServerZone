use std::collections::HashMap;

use super::command::*;

use crate::generated::proto_server::{
    BroadcastStream, PayloadTransformSync, Vector3, broadcast_stream,
};
use crate::proto_service::{
    client::backend_client::BackendClient, server::backent_server::BackendServerChannels,
};
use crate::{
    game::{contents::containts_director::ContaintsDirector, entity::entity::Entity},
    net::client::{self},
};

pub struct Zone {
    #[allow(dead_code)]
    name: String,
    id: u64,
    next_use_entity_id: u64,

    players: HashMap<u64, client::Cluster>,
    routeing_players: HashMap<u64, client::Cluster>,
    player_id_by_user_id: HashMap<u64, u64>,

    contains_directors: Vec<ContaintsDirector>,

    backend_client: BackendClient,
    backend_server_channels: BackendServerChannels,

    async_tasks: tokio::task::JoinSet<Option<CommandBox>>,
    zone_commands: Vec<CommandBox>,
}

impl Zone {
    pub fn new(
        name: String,
        backend_client: BackendClient,
        backend_server_channels: BackendServerChannels,
    ) -> Self {
        Zone {
            name,
            id: 0,
            next_use_entity_id: 0,

            players: HashMap::new(),
            routeing_players: HashMap::new(),
            player_id_by_user_id: HashMap::new(),

            contains_directors: Vec::new(),

            backend_client,
            backend_server_channels,

            async_tasks: tokio::task::JoinSet::new(),
            zone_commands: Vec::new(),
        }
    }

    pub async fn initialize(&mut self) {
        // コンテンツディレクターの初期化
        let mut director = ContaintsDirector::new();
        director.spawn_enemy(self.next_use_entity_id);
        self.next_use_entity_id += 1;
        self.contains_directors.push(director);
    }

    pub async fn update(&mut self) {
        // メッセージ処理
        self.execute_messages().await;

        // 通常更新処理
        self.players.iter_mut().for_each(|(_, cluster)| {
            cluster.update();
        });

        // コンテンツディレクターの処理
        self.contains_directors.iter_mut().for_each(|director| {
            director.update();
        });

        // 位置同期
        self.sync_entity_transform_all();
    }

    async fn execute_messages(&mut self) {
        // worldからのコマンド
        let world_command_len = self.backend_server_channels.world_route_receiver.len();
        let mut world_commands = Vec::with_capacity(world_command_len);
        self.backend_server_channels
            .world_route_receiver
            .recv_many(&mut world_commands, world_command_len)
            .await;
        world_commands.into_iter().for_each(|command| {
            command.execute(self);
        });

        // クライアントからのsyncコマンド
        let sync_message_len = self.backend_server_channels.sync_command_receiver.len();
        let mut sync_messages = Vec::with_capacity(sync_message_len);
        self.backend_server_channels
            .sync_command_receiver
            .recv_many(&mut sync_messages, sync_message_len)
            .await;
        sync_messages.into_iter().for_each(|command| {
            command.execute(self);
        });

        // クライアントからのコマンドを収集して実行
        self.execute_client_commands();

        // contents directorから
        let director_commands = self
            .contains_directors
            .iter_mut()
            .flat_map(|director| director.take_commands())
            .collect::<Vec<CommandBox>>();
        director_commands.into_iter().for_each(|command| {
            command.execute(self);
        });

        // zone commands
        let zone_commands = std::mem::take(&mut self.zone_commands);
        zone_commands.into_iter().for_each(|command| {
            command.execute(self);
        });

        // chainするタイプのコマンドを実行
        while let Some(result) = self.async_tasks.try_join_next() {
            if let Ok(Some(command)) = result {
                command.execute(self);
            }
        }
    }

    fn execute_client_commands(&mut self) {
        let commands: Vec<CommandBox> = self
            .players
            .values_mut()
            .flat_map(|cluster| cluster.take_commands())
            .collect();

        commands
            .into_iter()
            .for_each(|command| command.execute(self));
    }

    fn sync_entity_transform_all(&mut self) {
        let timestamp = chrono::Utc::now().timestamp_micros() as u64;

        let mut transforms = Vec::new();

        // プレイヤー
        self.players.iter().for_each(|(id, cluster)| {
            let position = cluster.player().position();
            transforms.push((*id, *position));
        });

        // 敵
        self.contains_directors.iter().for_each(|director| {
            director.enemies().iter().for_each(|(id, enemy)| {
                let position = enemy.position();
                transforms.push((*id, *position));
            });
        });

        transforms.into_iter().for_each(|(entity_id, position)| {
            let message = BroadcastStream {
                payload: Some(broadcast_stream::Payload::TransformSync(
                    PayloadTransformSync {
                        id: entity_id,
                        timestamp,
                        position: Some(Vector3 {
                            x: position.x,
                            y: position.y,
                            z: position.z,
                        }),
                    },
                )),
            };

            self.send_broadcast_message(message);
        });
    }

    pub fn send_broadcast_message(&mut self, message: BroadcastStream) {
        self.backend_server_channels
            .broadcast_senders
            .iter()
            .for_each(|sender| {
                let _ = sender.try_send(Ok(message.clone()));
            });
    }

    pub fn send_broadcast_message_to_gateway(&mut self, gateway_id: u64, message: BroadcastStream) {
        if let Some(sender) = self
            .backend_server_channels
            .broadcast_senders
            .get(&gateway_id)
        {
            sender.try_send(Ok(message)).unwrap_or_else(|e| {
                log::warn!("Failed to send message to gateway {}: {}", gateway_id, e)
            });
        } else {
            log::warn!(
                "Gateway with id {} not found when trying to send message",
                gateway_id
            );
        }
    }

    pub fn add_async_command(
        &mut self,
        task: impl std::future::Future<Output = Option<CommandBox>> + Send + 'static,
    ) {
        self.async_tasks.spawn(task);
    }

    pub fn add_zone_command(&mut self, command: CommandBox) {
        self.zone_commands.push(command);
    }

    // ---------- getter ----------
    pub fn players_mut(&mut self) -> &mut HashMap<u64, client::Cluster> {
        &mut self.players
    }

    pub fn routeing_players_mut(&mut self) -> &mut HashMap<u64, client::Cluster> {
        &mut self.routeing_players
    }

    pub fn player_id_by_user_id_mut(&mut self) -> &mut HashMap<u64, u64> {
        &mut self.player_id_by_user_id
    }

    #[allow(unused)]
    pub fn player_mut_by_user_id(&mut self, user_id: &u64) -> Option<&mut client::Cluster> {
        let player_id = self.player_id_by_user_id.get(user_id)?;
        self.players.get_mut(player_id)
    }

    #[allow(unused)]
    pub fn contains_director_mut(&mut self, index: usize) -> Option<&mut ContaintsDirector> {
        self.contains_directors.get_mut(index)
    }

    pub fn tonic_client_mut(&mut self) -> &mut BackendClient {
        &mut self.backend_client
    }

    pub fn entity_mut(&mut self, entity_id: &u64) -> Option<&mut dyn Entity> {
        if let Some(cluster) = self.players.get_mut(entity_id) {
            Some(cluster.player_mut())
        } else if let Some(enemy) = self
            .contains_directors
            .iter_mut()
            .find(|director| director.enemies().contains_key(entity_id))
            .and_then(|director| director.enemies_mut().get_mut(entity_id))
        {
            Some(enemy)
        } else {
            None
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn next_entity_id(&mut self) -> u64 {
        let id = self.next_use_entity_id;
        self.next_use_entity_id += 1;
        id
    }
}
