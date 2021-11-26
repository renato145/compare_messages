use anyhow::Result;
use axum::{body::Bytes, routing::post, Json, Router};
use compare_messages::{
    plot,
    proto_msg::{self, messager_server::MessagerServer},
    test_avro_axum_message, test_avro_zmq_message, test_grpc_message, test_grpc_zmq_message,
    test_json_message, test_zmq_json_message, JsonMessage, ServerGrpc, SCHEMA,
};
use env_logger::Env;
use log::{debug, info};
use prost::Message;
use std::net::SocketAddr;
use zeromq::{Socket, SocketRecv, SocketSend, ZmqMessage};

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

    info!("Comparing serialization formats over http and zmq:");
    info!("Sending {} messages...", n_tests);

    tokio::spawn(async {
        axum_server().await;
    });

    tokio::spawn(async {
        zmq_json_server().await;
    });

    tokio::spawn(async {
        zmq_avro_server().await;
    });

    tokio::spawn(async {
        grpc_server().await;
    });

    tokio::spawn(async {
        zmq_grpc_server().await;
    });

    let client = reqwest::Client::new();
    // let tests = [10, 50, 100];
    // let tests = [1, 10, 100, 500];
    let tests = [1, 10, 100, 500, 1000, 5000];
    // let tests = [1, 10, 100, 500, 1000, 5000, 10_000, 50_000];
    let mut results = Vec::new();

    for n in tests {
        let result = test_json_message(n_tests, &client, n).await?;
        println!("{}", result);
        results.push(result);
        let result = test_grpc_message(n_tests, n).await?;
        println!("{}", result);
        results.push(result);
        let result = test_avro_axum_message(n_tests, &client, n).await?;
        println!("{}", result);
        results.push(result);
        let result = test_zmq_json_message(n_tests, n).await?;
        println!("{}", result);
        results.push(result);
        let result = test_grpc_zmq_message(n_tests, n).await?;
        println!("{}", result);
        results.push(result);
        let result = test_avro_zmq_message(n_tests, n).await?;
        println!("{}", result);
        results.push(result);
        println!("-----------------------------------------------------------------------");
    }

    plot(results)?;
    Ok(())
}

async fn axum_server() {
    let app = Router::new()
        .route("/json", post(json_msg))
        .route("/avro", post(avro_msg));

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

async fn avro_msg(body: Bytes) -> Bytes {
    let value = avro_rs::Reader::new(&body[..])
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
        .unwrap();
    let decoded = avro_rs::from_value::<JsonMessage>(&value).unwrap();
    debug!("Server got: {:?}", decoded);
    let mut writer = avro_rs::Writer::new(&SCHEMA, Vec::new());
    writer.append_ser(&decoded).unwrap();
    let encoded = writer.into_inner().unwrap();
    Bytes::from_iter(encoded)
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

async fn zmq_json_server() {
    let mut socket = zeromq::RepSocket::new();
    let addr = "tcp://127.0.0.1:5000";
    info!("zmq json server starting...");
    socket.bind(addr).await.expect("Failed to connect");
    info!("zmq json listening on {}", addr);

    loop {
        let msg: String = socket.recv().await.unwrap().try_into().unwrap();
        let decoded = serde_json::from_str::<JsonMessage>(&msg).unwrap();
        let json_msg = serde_json::to_string(&decoded).unwrap();
        socket.send(json_msg.into()).await.unwrap();
    }
}

async fn zmq_grpc_server() {
    let mut socket = zeromq::RepSocket::new();
    let addr = "tcp://127.0.0.1:7000";
    socket.bind(addr).await.expect("Failed to connect");
    info!("zmq grpc listening on {}", addr);

    loop {
        let msg = socket.recv().await.unwrap();
        let bytes = msg
            .into_vecdeque()
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        let decoded = proto_msg::SomeMessage::decode(&bytes[..]).unwrap();
        debug!("Server got: {:?}", decoded);
        let mut encoded = Vec::new();
        decoded.encode(&mut encoded).unwrap();
        let encoded_msg = ZmqMessage::from(encoded);
        socket.send(encoded_msg).await.unwrap();
    }
}

async fn zmq_avro_server() {
    let mut socket = zeromq::RepSocket::new();
    let addr = "tcp://127.0.0.1:6000";
    socket.bind(addr).await.expect("Failed to connect");
    info!("zmq avro listening on {}", addr);

    loop {
        let msg = socket.recv().await.unwrap();
        let bytes = msg
            .into_vecdeque()
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let value = avro_rs::Reader::new(&bytes[..])
            .unwrap()
            .into_iter()
            .next()
            .unwrap()
            .unwrap();
        let decoded = avro_rs::from_value::<JsonMessage>(&value).unwrap();
        debug!("Server got: {:?}", decoded);
        let mut writer = avro_rs::Writer::new(&SCHEMA, Vec::new());
        writer.append_ser(&decoded).unwrap();
        let encoded = writer.into_inner().unwrap();
        let encoded_msg = ZmqMessage::from(encoded);
        socket.send(encoded_msg).await.unwrap();
    }
}
