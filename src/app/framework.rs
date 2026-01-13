use log;
use std::{net::Ipv4Addr, time::Duration};
use ticktock;
use tokio::net::TcpListener;

use super::tick_time;

use crate::{game::action::action_list_table, zone::zone::Zone};

pub async fn run() {
    // 初期化
    let tcp_listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, 3215)).await;
    if tcp_listener.is_err() {
        log::error!("Error binding TCP listener: {}", tcp_listener.unwrap_err());
        return;
    }
    // ActionListTableをDBから読む
    let action_list_table = action_list_table::ActionListTable::load_from_database();

    // Zoneの生成と初期化
    let mut zone = Zone::new(
        "TestZone".to_string(),
        tcp_listener.unwrap(),
        &action_list_table,
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
