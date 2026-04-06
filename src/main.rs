mod app;
mod game;
mod generated;
mod math;
mod net;
mod proto_service;
mod zone;

#[tokio::main]
async fn main() {
    app::log_init::init();

    log::info!("Server starting...\n");

    log::info!("Build info");
    log::info!(
        "Package version: {}, OS: {}, Arch: {}\n",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    app::framework::run().await;
}
