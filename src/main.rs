use anyhow::Result;
use axum::{routing::post, Json, Router};
use compare_messages::{
    proto_msg::messager_server::MessagerServer, test_grpc_message, test_json_message, JsonMessage,
    ServerGrpc,
};
use env_logger::Env;
use log::{debug, info};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let mut args = std::env::args();
    args.next();
    let n_tests: usize = args
        .next()
        .unwrap_or_else(|| "100".to_string())
        .parse()
        .unwrap();

    info!("Comparing serialization formats over http:");
    info!("Sending {} messages...", n_tests);

    tokio::spawn(async {
        axum_server().await;
    });

    tokio::spawn(async {
        grpc_server().await;
    });

    let client = reqwest::Client::new();
    let tests = [10, 100, 1000, 5000];

    for n in tests {
        let result = test_json_message(n_tests, &client, n).await?;
        println!("{}", result);
        let result = test_grpc_message(n_tests, n).await?;
        println!("{}", result);
    }

    Ok(())
}

async fn axum_server() {
    let app = Router::new().route("/json", post(json_msg));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("axum listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn json_msg(Json(message): Json<JsonMessage>) -> Json<JsonMessage> {
    debug!("Server got: {:?}", message);
    Json(message)
}

async fn grpc_server() {
    let addr = "[::1]:4000".parse().unwrap();
    let greeter = ServerGrpc::default();
    info!("grpc listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(MessagerServer::new(greeter))
        .serve(addr)
        .await
        .unwrap();
}
