use etcd_client::{PutOptions, WatchOptions};
use log;
use std::{env, net::Ipv4Addr, time::Duration};
use ticktock;

use super::tick_time;

use crate::{
    game::action::action_list_table,
    generated::proto_client::zone_broadcast_service_client::ZoneBroadcastServiceClient,
    proto_service::{client::backend_client::BackendClient, server::backent_server},
    zone::zone::Zone,
};

pub async fn run() {
    // 初期化
    let server_ip = env::var("LISTEN_ADDR").unwrap_or_else(|_| Ipv4Addr::LOCALHOST.to_string());
    let port = env::var("PORT")
        .unwrap_or_else(|_| "50053".into())
        .parse()
        .unwrap_or(50053u16);
    let zone_id = env::var("ZONE_ID").unwrap_or_else(|_| "0".into());

    // serverの起動
    let backend_server_receiver = backent_server::create_backend_server_receiver(
        env::var("COMMAND_CHANNEL_SIZE")
            .unwrap_or_else(|_| "128".into())
            .parse()
            .unwrap_or(128),
        port,
    )
    .await;

    // 各種サーバーへの接続
    let backend_client = BackendClient::new().await;

    // etcdにゾーンの情報を登録
    let etcd_client = etcd_client::Client::connect(
        [format!(
            "{}:2379",
            env::var("ETCD_ADDR").unwrap_or_else(|_| "localhost".into())
        )],
        None,
    )
    .await
    .expect("Failed to connect to etcd server");
    let mut etcd_client_checker = etcd_client.clone();
    let mut etcd_client_keeper = etcd_client.clone();

    // ゾーンの情報をetcdに登録し、定期的に更新する
    tokio::spawn(async move {
        let lease = etcd_client_keeper
            .lease_grant(5, None)
            .await
            .expect("Failed to create etcd lease");
        let options = PutOptions::new().with_lease(lease.id());
        etcd_client_keeper
            .put(
                format!("zones/{}", zone_id),
                format!("{}:{}", server_ip, port),
                Some(options),
            )
            .await
            .expect("Failed to put zone info into etcd");

        loop {
            etcd_client_keeper
                .lease_keep_alive(lease.id())
                .await
                .expect("Failed to keep alive etcd lease");
            log::info!("Updated zone info in etcd with lease ID {}", lease.id());

            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });

    // gatewayサーバーの情報を取得
    let backend_client_clone = backend_client.clone();
    tokio::spawn(async move {
        let prefix = "gateways/";
        let config = WatchOptions::new().with_prefix();
        let mut stream = etcd_client_checker
            .watch(prefix, Some(config))
            .await
            .expect("Failed to watch etcd for gateway changes");
        loop {
            match stream.message().await {
                Ok(Some(response)) => {
                    for event in response.events() {
                        match event.event_type() {
                            etcd_client::EventType::Put => {
                                if let Some(kv) = event.kv() {
                                    let gateway_id = String::from_utf8_lossy(kv.key())
                                        .strip_prefix(prefix)
                                        .unwrap_or_default()
                                        .parse::<u64>()
                                        .unwrap_or_default();
                                    let url = String::from_utf8_lossy(kv.value())
                                        .parse::<String>()
                                        .unwrap_or_default();

                                    let Ok(client) = ZoneBroadcastServiceClient::connect(format!(
                                        "http://{}",
                                        url
                                    ))
                                    .await
                                    else {
                                        log::error!("Failed to connect to gateway at {}", url);
                                        continue;
                                    };

                                    backend_client_clone
                                        .add_gateway_client(gateway_id, client)
                                        .await;

                                    log::info!("Added gateway client: {} -> {}", gateway_id, url);
                                }
                            }
                            etcd_client::EventType::Delete => {
                                if let Some(kv) = event.kv() {
                                    let gateway_id = String::from_utf8_lossy(kv.key())
                                        .strip_prefix(prefix)
                                        .unwrap_or_default()
                                        .parse::<u64>()
                                        .unwrap_or_default();

                                    backend_client_clone.remove_gateway_client(gateway_id).await;
                                    log::info!("Removed gateway client: {}", gateway_id);
                                }
                            }
                        }
                    }
                }
                _ => {
                    log::error!("Failed to receive etcd watch message");
                }
            }
        }
    });

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
