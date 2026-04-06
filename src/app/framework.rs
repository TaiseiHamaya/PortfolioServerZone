use log;
use std::time::Duration;
use ticktock;

use super::tick_time;

use crate::{
    game::action::action_list_table,
    proto_service::{client::backend_client::BackendClient, server::backent_server},
    zone::zone::Zone,
};

pub async fn run() {
    // 初期化
    // バックエンドサーバーと接続
    let backend_client = BackendClient::new().await;

    // serverの起動
    let backend_server_receiver = backent_server::create_backend_server(0).await;

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
