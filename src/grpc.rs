use crate::TestResult;
use anyhow::Result;
use fake::{faker::lorem::en::Words, Fake};
use log::debug;
use prost::Message;
use proto_msg::messager_client::MessagerClient;
use proto_msg::messager_server::Messager;
use proto_msg::SomeMessage;
use std::time::Instant;
use tonic::{Request, Response, Status};
use zeromq::{Socket, SocketRecv, SocketSend, ZmqMessage};

pub mod proto_msg {
    tonic::include_proto!("message");
}

#[derive(Debug, Default)]
pub struct ServerGrpc {}

#[tonic::async_trait]
impl Messager for ServerGrpc {
    async fn send_message(
        &self,
        request: Request<SomeMessage>,
    ) -> Result<Response<SomeMessage>, Status> {
        let reply = request.into_inner();
        debug!("Got {:?}", reply);
        Ok(Response::new(reply))
    }
}

pub async fn test_grpc_message(n_tests: usize, num_elements: usize) -> Result<TestResult> {
    let mut client = MessagerClient::connect("http://[::1]:4000").await?;

    let msg = SomeMessage {
        values: fake::vec![f64; num_elements],
        descriptions: Words(num_elements..num_elements + 1).fake(),
    };

    let t0 = Instant::now();
    for _ in 0..n_tests {
        let request = tonic::Request::new(msg.clone());
        let response = client.send_message(request).await?.into_inner();
        debug!("Client got: {:?}", response);
    }
    let grpc_result = TestResult::new("Tonic grpc", n_tests, num_elements, t0.elapsed());
    Ok(grpc_result)
}

pub async fn test_grpc_zmq_message(n_tests: usize, num_elements: usize) -> Result<TestResult> {
    let msg = SomeMessage {
        values: fake::vec![f64; num_elements],
        descriptions: Words(num_elements..num_elements + 1).fake(),
    };

    let mut socket = zeromq::ReqSocket::new();
    debug!("ZMQ json client connecting...");
    socket.connect("tcp://127.0.0.1:7000").await?;
    debug!("ZMQ json client connected");
    let t0 = Instant::now();
    for _ in 0..n_tests {
        let mut encoded = Vec::new();
        msg.encode(&mut encoded)?;
        let encoded_msg = ZmqMessage::from(encoded);
        socket.send(encoded_msg).await?;

        let bytes = socket
            .recv()
            .await?
            .into_vecdeque()
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        let decoded = SomeMessage::decode(&bytes[..])?;
        debug!("Client got: {:?}", decoded);
    }
    let grpc_result = TestResult::new("Zmq grpc  ", n_tests, num_elements, t0.elapsed());
    Ok(grpc_result)
}
