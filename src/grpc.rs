use anyhow::Result;
use fake::{faker::lorem::en::Words, Fake};
use log::debug;
use proto_msg::messager_client::MessagerClient;
use proto_msg::messager_server::Messager;
use proto_msg::Message;
use std::time::Instant;
use tonic::{Request, Response, Status};

use crate::TestResult;

pub mod proto_msg {
    tonic::include_proto!("message");
}

#[derive(Debug, Default)]
pub struct ServerGrpc {}

#[tonic::async_trait]
impl Messager for ServerGrpc {
    async fn send_message(&self, request: Request<Message>) -> Result<Response<Message>, Status> {
        debug!("Got {:?}", request);
        let reply = request.into_inner();
        Ok(Response::new(reply))
    }
}

pub async fn test_grpc_message(n_tests: usize, num_elements: usize) -> Result<TestResult> {
    let mut client = MessagerClient::connect("http://[::1]:4000").await?;

    let msg = Message {
        values: fake::vec![f64; num_elements],
        descriptions: Words(num_elements..num_elements + 1).fake(),
    };

    let t0 = Instant::now();
    for _ in 0..n_tests {
        let request = tonic::Request::new(msg.clone());
        let response = client.send_message(request).await?;
        debug!("Client got: {:?}", response);
    }
    let grpc_result = TestResult::new("Tonic grpc", n_tests, num_elements, t0.elapsed());
    Ok(grpc_result)
}
