use lazy_static::lazy_static;

use std::sync::RwLock;
use chrono::{DateTime, Utc};

lazy_static! {
    static ref TICK_TIME: RwLock<DateTime<Utc>> = RwLock::new(Utc::now());
}

pub fn update_tick_time() {
    let mut tick_time = TICK_TIME.write().unwrap_or_else(|poison| poison.into_inner());
    *tick_time = Utc::now();
}

pub fn get_tick_time() -> DateTime<Utc> {
    TICK_TIME.read().unwrap_or_else(|poison| poison.into_inner()).clone()
}