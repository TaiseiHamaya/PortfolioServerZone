use std::collections::HashMap;
use std::task::{Context, Poll};

use futures::{StreamExt, stream};
use log;
use tokio::net::TcpListener;

use nalgebra::Point3;
use protobuf::*;

use super::{command::*, zone_request_cache::ZoneRequestChash};

use crate::{
    game::{
        action::action_list_table,
        contents::containts_director::ContaintsDirector,
        entity::{entity::Entity, player::Player},
    },
    net::{
        client::{self, TcpClient},
        proto,
    },
};

pub struct Zone<'action_list> {
    name: String,
    next_use_entity_id: u64,
    players: HashMap<u64, client::Cluster<'action_list>>,

    containts_directors: Vec<ContaintsDirector>,

    tcp_listener: TcpListener,

    actions_list_table: &'action_list action_list_table::ActionListTable,
    zone_request_chash: ZoneRequestChash<'action_list>,
}

impl<'action_list> Zone<'action_list> {
    pub fn new(
        name: String,
        listner: TcpListener,
        actions_list_table: &'action_list action_list_table::ActionListTable,
    ) -> Self {
        Zone {
            name,
            next_use_entity_id: 0,
            players: HashMap::new(),

            containts_directors: Vec::new(),

            tcp_listener: listner,

            actions_list_table,
            zone_request_chash: ZoneRequestChash::new(),
        }
    }

    pub async fn initialize(&mut self) {
        // コンテンツディレクターの初期化
        let mut director = ContaintsDirector::new();
        director.spawn_enemy(self.next_use_entity_id);
        self.next_use_entity_id += 1;
        self.containts_directors.push(director);
    }

    pub async fn update(&mut self) {
        // パケット受信
        self.recv_all().await;

        // クライアント接続受付
        self.accept_client().await;

        // クライアントのパケット処理
        self.players.iter_mut().for_each(|(_, client)| {
            client.process_packets();
        });

        // 通常更新処理
        self.players.iter_mut().for_each(|(_, player)| {
            player.update();
        });

        // コンテンツディレクターの処理
        self.containts_directors.iter_mut().for_each(|director| {
            director.update();
        });

        // コマンド処理
        self.execute_client_commands();

        // 位置同期
        self.sync_entity_transform_all();

        // クライアント追加/削除処理
        self.add_client_accepted();
        self.remove_client_chashed();

        // チャッシュクリア
        self.zone_request_chash.clear();

        // パケット送信
        self.send_all().await;
    }

    async fn recv_all(&mut self) {
        stream::iter(self.players.values_mut())
            .for_each_concurrent(None, |client| async move {
                client.recv_packets().await;
            })
            .await;
    }

    async fn accept_client(&mut self) {
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        match self.tcp_listener.poll_accept(&mut cx) {
            Poll::Ready(Ok((stream, addr))) => {
                // 後でDB接続に変える
                log::info!("Accepted connection from {}", addr);
                let player_id = self.next_use_entity_id;
                self.next_use_entity_id += 1;
                let player_name = format!("Player{}", player_id);

                let position = Point3::new(0.0, 0.0, 0.0); // 初期位置は(0, 0, 0)
                let player = Player::new(
                    player_id,
                    position,
                    10000,
                    self.actions_list_table.get_action_list(0).unwrap(),
                );
                let client_cluster =
                    client::Cluster::new(player, player_name.clone(), TcpClient::new(stream, addr));

                // ログイン要求をチャッシュに追加
                self.zone_request_chash.push_login(client_cluster);
            }
            Poll::Ready(Err(e)) => {
                log::error!("Accept error: {} (Zone: {})", e, self.name);
            }
            Poll::Pending => {}
        }
    }

    async fn send_all(&mut self) {
        self.players.iter_mut().for_each(|(_, client)| {
            client.send_packets();
        });
    }

    // プレイヤー追加
    fn add_client_accepted(&mut self) {
        let login_chash = self.zone_request_chash.get_login_chash_take();
        login_chash.into_iter().for_each(
            |mut login: super::zone_request_cache::ZonePlayerLogin<'_>| {
                // 接続完了通知
                login.client_cluster.on_accepted();
                // 既存プレイヤーの情報を送信
                self.players.iter_mut().for_each(|(id, cluster)| {
                    let mut packet = proto::Packet::new();
                    packet
                        .set_category_login_message(proto::CategoryLoginMessage::LoginNotification);
                    let mut body = proto::PayloadLoginNotification::new();
                    body.set_id(*id);
                    body.set_username(cluster.player_name().clone());
                    let mut position = proto::Vector3::new();
                    let cluster_position = cluster.player().position();
                    position.set_x(cluster_position.x);
                    position.set_y(cluster_position.y);
                    position.set_z(cluster_position.z);
                    body.set_position(position);
                    packet.set_payload(body.serialize().unwrap());
                    // パケットを積む
                    login.client_cluster.stack_packet(packet);
                });
                // 敵のスポーン情報を送信
                self.containts_directors[0]
                    .get_enemies_mut()
                    .iter()
                    .for_each(|(_, enemy)| {
                        let mut packet = proto::Packet::new();
                        packet.set_category_enemy_message(proto::CategoryEnemyMessage::EnemySpawn);
                        let mut body = proto::PayloadEnemySpawn::new();
                        body.set_id(enemy.id());
                        body.set_name(enemy.get_name().clone());
                        let mut position = proto::Vector3::new();
                        let enemy_position = enemy.position();
                        position.set_x(enemy_position.x);
                        position.set_y(enemy_position.y);
                        position.set_z(enemy_position.z);
                        body.set_position(position);
                        let payload = body.serialize();
                        if payload.is_err() {
                            log::error!(
                                "Failed to serialize enemy spawn packet: {}",
                                payload.unwrap_err()
                            );
                            return;
                        }
                        packet.set_payload(payload.unwrap());
                        login.client_cluster.stack_packet(packet);
                    });
                // プレイヤーリストに追加
                self.players.insert(login.id, login.client_cluster);
            },
        );
    }

    fn execute_client_commands(&mut self) {
        let commands: Vec<Box<dyn CommandTrait>> = self
            .players
            .values_mut()
            .flat_map(|cluster| cluster.take_commands())
            .collect();
        for command in commands {
            command.execute(self);
        }
    }

    // アプリケーション内での削除処理
    fn remove_client_chashed(&mut self) {
        let logout_chash = self.zone_request_chash.get_logout_chash_take();

        logout_chash.into_iter().for_each(|logout| {
            self.players.remove(&logout.entity_id);
        });
    }

    pub fn sync_entity_transform_all(&mut self) {
        let timestamp = chrono::Utc::now().timestamp_micros() as u64;
        let transforms = self
            .players
            .iter()
            .map(|(id, cluster)| {
                let position = cluster.player().position();
                (*id, timestamp, *position)
            })
            .collect::<Vec<_>>();
        transforms
            .into_iter()
            .for_each(|(entity_id, timestamp, position)| {
                self.sync_entity_transform(entity_id, timestamp, position);
            });
    }

    // 位置の同期をクライアントに通知
    pub fn sync_entity_transform(&mut self, entity_id: u64, timestamp: u64, position: Point3<f32>) {
        let mut packet = proto::Packet::new();
        packet.set_category_sync_message(proto::CategorySyncMessage::SyncTransform);
        let mut body = proto::PayloadTransformSync::new();
        body.set_id(entity_id);
        body.set_timestamp(timestamp);
        let mut pos = proto::Vector3::new();
        pos.set_x(position.x);
        pos.set_y(position.y);
        pos.set_z(position.z);
        body.set_position(pos);
        let payload = body.serialize();
        if payload.is_err() {
            return;
        }
        packet.set_payload(payload.unwrap());
        self.players.iter_mut().for_each(|(player_id, cluster)| {
            if entity_id == *player_id {
                return; // 自分には送らない
            }
            cluster.stack_packet(packet.clone());
        });
    }

    // ---------- getter ----------
    pub fn players_mut(&mut self) -> &mut HashMap<u64, client::Cluster<'action_list>> {
        &mut self.players
    }

    pub fn zone_request_chash_mut(&mut self) -> &mut ZoneRequestChash<'action_list> {
        &mut self.zone_request_chash
    }

    pub fn containts_director_mut(&mut self, index: usize) -> Option<&mut ContaintsDirector> {
        self.containts_directors.get_mut(index)
    }
}
