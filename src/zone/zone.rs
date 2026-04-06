use log;
use nalgebra::Point3;
use std::collections::HashMap;

use super::{command::*, zone_request_cache::ZoneRequestChash};

use crate::generated::proto_client::{PayloadTransformSync, Vector3};
use crate::proto_service::{
    client::backend_client::BackendClient, server::backent_server::BackendServerReceiver,
};
use crate::{
    game::{contents::containts_director::ContaintsDirector, entity::entity::Entity},
    net::client::{self},
};

pub struct Zone {
    name: String,
    next_use_entity_id: u64,

    players: HashMap<u64, client::Cluster>,

    contains_directors: Vec<ContaintsDirector>,

    zone_request_chash: ZoneRequestChash,

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
        Zone {
            name,
            next_use_entity_id: 0,
            players: HashMap::new(),

            contains_directors: Vec::new(),

            zone_request_chash: ZoneRequestChash::new(),

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
        self.receive_messages();

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

        // クライアント追加/削除処理
        self.add_client_accepted();
        self.remove_client_cashed();

        // チャッシュクリア
        self.zone_request_chash.clear();
    }

    fn receive_messages(&mut self) {
        // クライアントからのsyncコマンド
        let sync_message_len = self.backend_server_receiver.sync_command_receiver.len();
        let mut sync_messages = Vec::with_capacity(sync_message_len);
        self.backend_server_receiver
            .sync_command_receiver
            .blocking_recv_many(&mut sync_messages, sync_message_len);
        sync_messages.into_iter().for_each(|message| {
            message.execute(self);
        });
        // worldからのコマンド
        let world_command_len = self.backend_server_receiver.world_command_receiver.len();
        let mut world_commands = Vec::with_capacity(world_command_len);
        self.backend_server_receiver
            .world_command_receiver
            .blocking_recv_many(&mut world_commands, world_command_len);
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

    // プレイヤー追加
    fn add_client_accepted(&mut self) {
        let login_chash = self.zone_request_chash.get_login_chash_take();
        login_chash.into_iter().for_each(|login| {
            // 接続完了通知
            // プレイヤーリストに追加
            self.players.insert(login.id, login.client_cluster);
        });
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

    // アプリケーション内での削除処理
    fn remove_client_cashed(&mut self) {
        let logout_chash = self.zone_request_chash.get_logout_chash_take();

        logout_chash.into_iter().for_each(|logout| {
            self.players.remove(&logout.entity_id);
        });
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
        let mut client = self.backend_client.zone_broadcast_service_client.clone();
        let message = PayloadTransformSync {
            id: entity_id,
            timestamp,
            position: Some(Vector3 {
                x: position.x,
                y: position.y,
                z: position.z,
            }),
        };
        tokio::spawn(async move {
            if let Err(e) = client.sync_transform(message).await {
                log::error!("Failed to sync entity transform: {}", e);
            }
        });
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

    pub fn zone_request_chash_mut(&mut self) -> &mut ZoneRequestChash {
        &mut self.zone_request_chash
    }

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
}
