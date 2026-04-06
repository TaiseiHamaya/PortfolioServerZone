use std::fs;

use tonic_prost_build;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var("PROTOC", "./portfolio-proto/protoc-32.1-win64/bin/protoc");
    }

    // 出力ファイル・ディレクトリの作成
    let server_out_dir = "src/generated/server/";
    let client_out_dir = "src/generated/client/";
    fs::create_dir_all(server_out_dir)?;
    fs::create_dir_all(client_out_dir)?;

    // サーバーコードの生成
    tonic_prost_build::configure()
        .build_client(false)
        .build_server(true)
        .out_dir(server_out_dir)
        .type_attribute("SessionId", "#[derive(PartialOrd, Ord)]")
        .compile_protos(
            &[
                "process/zone/command_sync.proto",
                "process/world/service_zone.proto",
            ],
            &["portfolio-proto"],
        )?;

    tonic_prost_build::configure()
        .build_client(true)
        .build_server(false)
        .out_dir(client_out_dir)
        .type_attribute("SessionId", "#[derive(PartialOrd, Ord)]")
        .compile_protos(
            &[
                "process/zone/event_sync.proto",
                "process/world/command_zone.proto",
                "process/db/record/service.proto",
            ],
            &["portfolio-proto"],
        )?;

    Ok(())
}
