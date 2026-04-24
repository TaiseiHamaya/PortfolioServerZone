use dotenvy::dotenv;

mod app;
mod game;
mod generated;
mod logger;
mod math;
mod net;
mod proto_service;
mod zone;

#[tokio::main]
async fn main() {
    logger::init().expect("Failed to initialize logger");

    log::info!("Server starting...\n");

    log::info!("Build info");
    log::info!(
        "Package version: {}, OS: {}, Arch: {}\n",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    log::info!("Loading env variables...");
    dotenv().ok();

    app::framework::run().await;
}
