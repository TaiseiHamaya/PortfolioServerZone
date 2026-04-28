use log;
use nalgebra::Point3;
use std::collections::HashMap;

use super::command::*;

use crate::generated::proto_client::{PayloadTransformSync, Vector3};
use crate::proto_service::{
    client::backend_client::BackendClient, server::backent_server::BackendServerReceiver,
};
use crate::zone::gateway_clients::GatewayClients;
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

    gateway_clients: GatewayClients,

    backend_client: BackendClient,
    backend_server_receiver: BackendServerReceiver,

    async_tasks: tokio::task::JoinSet<Option<CommandBox>>,
}

impl Zone {
    pub fn new(
        name: String,
        backend_client: BackendClient,
        backend_server_receiver: BackendServerReceiver,
    ) -> Self {
        let channels = backend_client.zone_broadcast_service_clients.clone();
        Zone {
            name,
            id: 0,
            next_use_entity_id: 0,

            players: HashMap::new(),
            routeing_players: HashMap::new(),
            player_id_by_user_id: HashMap::new(),

            contains_directors: Vec::new(),

            gateway_clients: GatewayClients::new(channels),

            backend_client,
            backend_server_receiver,

            async_tasks: tokio::task::JoinSet::new(),
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
        self.receive_messages().await;

        // 通常更新処理
        self.players.iter_mut().for_each(|(_, cluster)| {
            cluster.update();
        });

        // コンテンツディレクターの処理
        self.contains_directors.iter_mut().for_each(|director| {
            director.update();
        });

        // コマンド処理
        self.execute_client_commands();

        // 位置同期
        self.sync_entity_transform_all();
    }

    async fn receive_messages(&mut self) {
        // クライアントからのsyncコマンド
        let sync_message_len = self.backend_server_receiver.sync_command_receiver.len();
        let mut sync_messages = Vec::with_capacity(sync_message_len);
        self.backend_server_receiver
            .sync_command_receiver
            .recv_many(&mut sync_messages, sync_message_len)
            .await;
        sync_messages.into_iter().for_each(|message| {
            message.execute(self);
        });
        // worldからのコマンド
        let world_command_len = self.backend_server_receiver.world_route_receiver.len();
        let mut world_commands = Vec::with_capacity(world_command_len);
        self.backend_server_receiver
            .world_route_receiver
            .recv_many(&mut world_commands, world_command_len)
            .await;
        world_commands.into_iter().for_each(|command| {
            command.execute(self);
        });

        // backendから
        let sync_command_len = self.backend_server_receiver.sync_command_receiver.len();
        let mut sync_commands = Vec::with_capacity(sync_command_len);
        self.backend_server_receiver
            .sync_command_receiver
            .recv_many(&mut sync_commands, sync_command_len)
            .await;
        sync_commands.into_iter().for_each(|command| {
            command.execute(self);
        });
        let world_command_len = self.backend_server_receiver.world_route_receiver.len();
        let mut world_commands = Vec::with_capacity(world_command_len);
        self.backend_server_receiver
            .world_route_receiver
            .recv_many(&mut world_commands, world_command_len)
            .await;
        world_commands.into_iter().for_each(|command| {
            command.execute(self);
        });

        // クライアントからのコマンドを収集して実行
        let commnads = self
            .players
            .values_mut()
            .flat_map(|cluster| cluster.take_commands())
            .collect::<Vec<CommandBox>>();
        commnads
            .into_iter()
            .for_each(|command| command.execute(self));

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

    pub fn sync_entity_transform_all(&mut self) {
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
            self.sync_entity_transform(entity_id, timestamp, position);
        });
    }

    // 位置の同期をクライアントに通知
    pub fn sync_entity_transform(&mut self, entity_id: u64, timestamp: u64, position: Point3<f32>) {
        let clients = self.gateway_clients.clients_vec();
        let message = PayloadTransformSync {
            id: entity_id,
            timestamp,
            position: Some(Vector3 {
                x: position.x,
                y: position.y,
                z: position.z,
            }),
        };
        for mut client in clients {
            let message_clone = message.clone();
            tokio::spawn(async move {
                if let Err(e) = client.sync_transform(message_clone).await {
                    log::error!("Failed to sync entity transform: {}", e);
                }
            });
        }
    }

    pub fn add_async_command(
        &mut self,
        task: impl std::future::Future<Output = Option<CommandBox>> + Send + 'static,
    ) {
        self.async_tasks.spawn(task);
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

    pub fn player_mut_by_user_id(&mut self, user_id: &u64) -> Option<&mut client::Cluster> {
        let player_id = self.player_id_by_user_id.get(user_id)?;
        self.players.get_mut(player_id)
    }

    #[allow(dead_code)]
    pub fn contains_director_mut(&mut self, index: usize) -> Option<&mut ContaintsDirector> {
        self.contains_directors.get_mut(index)
    }

    pub fn tonic_client_mut(&mut self) -> &mut BackendClient {
        &mut self.backend_client
    }

    pub fn gateway_clients(&self) -> &GatewayClients {
        &self.gateway_clients
    }

    pub fn gateway_clients_mut(&mut self) -> &mut GatewayClients {
        &mut self.gateway_clients
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
