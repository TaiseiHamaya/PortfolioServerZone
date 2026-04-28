use log;
use std::{
    env,
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use ticktock;
use tonic::transport::{Endpoint, channel};

use super::tick_time;

use crate::{
    ec2_helper, etcd_client_helper,
    game::action::action_list_table,
    proto_service::{client::backend_client::BackendClient, server::backent_server},
    zone::zone::Zone,
};

pub async fn run() {
    // 初期化
    let server_address = ec2_helper::get_local_ip().await;
    let port = env::var("PORT")
        .unwrap_or_else(|_| "50053".into())
        .parse()
        .unwrap_or(50053u16);
    let zone_id = env::var("ZONE_ID").unwrap_or_else(|_| "0".into());
    let listen_endpoint = SocketAddr::new(IpAddr::V4(server_address), port);

    // grpc serverの起動
    let backend_server_receiver = backent_server::create_grpc_service(
        env::var("COMMAND_CHANNEL_SIZE")
            .unwrap_or_else(|_| "128".into())
            .parse()
            .unwrap_or(128),
        port,
    )
    .await;

    // etcd clientと接続
    let etcd_client = etcd_client::Client::connect(
        [format!(
            "{}:2379",
            env::var("ETCD_ADDR").unwrap_or_else(|_| "localhost".into())
        )],
        None,
    )
    .await
    .expect("Failed to connect to etcd server");

    let db_server_addr =
        etcd_client_helper::get_existing_service_endpoint(etcd_client.clone(), "db".to_string())
            .await
            .map(|kv| ["http://", &String::from_utf8_lossy(kv.value())].concat())
            .unwrap_or_else(|| format!("http://127.0.0.1:{}", 50050));

    let world_server_addr =
        etcd_client_helper::get_existing_service_endpoint(etcd_client.clone(), "world".to_string())
            .await
            .map(|kv| ["http://", &String::from_utf8_lossy(kv.value())].concat())
            .unwrap_or_else(|| format!("http://127.0.0.1:{}", 50051));

    // 各種サーバーへの接続
    let backend_client = BackendClient::new(
        Endpoint::from_shared(world_server_addr).expect("Invalid world server address"),
        Endpoint::from_shared(db_server_addr).expect("Invalid DB server address"),
    )
    .await;

    let _lease_task = etcd_client_helper::register_service_endpoint(
        etcd_client.clone(),
        listen_endpoint,
        format!("zones/{}", zone_id),
    )
    .await;

    let broadcast_service_clients = backend_client.zone_broadcast_service_clients.clone();
    etcd_client_helper::watch_changes(etcd_client.clone(), "gateways/", move |event| {
        let Some(kv) = event.kv() else {
            log::warn!("Received watch event without KV: {:?}", event);
            return;
        };
        let gateway_id = match String::from_utf8_lossy(kv.key())
            .strip_prefix("gateways/")
            .map(|id_str| id_str.parse::<u64>())
        {
            Some(Ok(id)) => id,
            _ => {
                log::warn!(
                    "Invalid gateway key format: {}",
                    String::from_utf8_lossy(kv.key())
                );
                return;
            }
        };
        let uri = ["http://", &String::from_utf8_lossy(kv.value())].concat();

        match event.event_type() {
            etcd_client::EventType::Put => {
                log::info!("Gateway added/updated: ID={}, URI={}", gateway_id, uri);
                let channel = match channel::Endpoint::from_shared(uri.clone())
                    .map(|endpoint| endpoint.connect_lazy())
                {
                    Ok(ch) => ch,
                    Err(e) => {
                        log::error!("Failed to create channel for gateway {}: {}", gateway_id, e);
                        return;
                    }
                };
                broadcast_service_clients.insert(gateway_id, channel);
            }
            etcd_client::EventType::Delete => {
                log::info!("Gateway removed: ID={}", gateway_id);
                broadcast_service_clients.remove(&gateway_id);
            }
        }
    })
    .await;

    let broadcast_service_clients = backend_client.zone_broadcast_service_clients.clone();
    etcd_client_helper::get_existing_service_endpoint_prefix(
        etcd_client.clone(),
        Some("zones/"),
        move |key, value| {
            log::info!(
                "Received zone endpoint update: key={}, value={}",
                key,
                value
            );
            let zone_id = match key.strip_prefix("zones/").map(|v| v.parse::<u64>()) {
                Some(Ok(id)) => id,
                _ => {
                    log::warn!("Invalid zone key format: {}", key);
                    return;
                }
            };
            let uri = ["http://", &value].concat();
            let channel =
                match channel::Endpoint::from_shared(uri).map(|endpoint| endpoint.connect_lazy()) {
                    Ok(ch) => ch,
                    Err(e) => {
                        log::error!("Failed to create channel for zone {}: {}", zone_id, e);
                        return;
                    }
                };
            broadcast_service_clients.insert(zone_id, channel);
        },
    )
    .await;

    // ActionListTableをDBから読む
    action_list_table::ActionListTable::load_from_database();

    // Zoneの生成と初期化
    let mut zone = Zone::new(
        "TestZone".to_string(),
        backend_client,
        backend_server_receiver,
    );
    zone.initialize().await;
    let tick_duration = chrono::TimeDelta::from_std(Duration::from_millis(50)).unwrap();
    let ticktock = ticktock::Clock::new(tick_duration.to_std().unwrap());

    for (tick, _) in ticktock.iter() {
        tick_time::update_tick_time();

        let tick_time = tick_time::get_tick_time();
        log::info!("Tick {} at {}", tick, tick_time);

        zone.update().await;

        let end = chrono::Utc::now();

        if end - tick_time > tick_duration {
            log::warn!(
                "Tick {} is running behind schedule! Time-'{}ms'",
                tick,
                (end - tick_time).to_std().unwrap().as_millis()
            );
        }
    }

    // 終了処理
}
