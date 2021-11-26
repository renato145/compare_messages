use crate::{JsonMessage, TestResult};
use anyhow::Result;
use fake::{faker::lorem::en::Words, Fake};
use log::debug;
use std::time::Instant;
use zeromq::{Socket, SocketRecv, SocketSend};

pub async fn test_zmq_json_message(n_tests: usize, num_elements: usize) -> Result<TestResult> {
    let msg = JsonMessage {
        values: fake::vec![f64; num_elements],
        descriptions: Words(num_elements..num_elements + 1).fake(),
    };

    let mut socket = zeromq::ReqSocket::new();
	debug!("ZMQ json client connecting...");
    socket.connect("tcp://127.0.0.1:5000").await?;
	debug!("ZMQ json client connected");
    let t0 = Instant::now();
    for _ in 0..n_tests {
        let json_msg = serde_json::to_string(&msg)?;
        socket.send(json_msg.into()).await?;
        let received: String = socket.recv().await?.try_into().unwrap();
        let decoded = serde_json::from_str::<JsonMessage>(&received)?;
        debug!("Client got: {:?}", decoded);
    }
    let json_result = TestResult::new("Zmq json  ", n_tests, num_elements, t0.elapsed());
    Ok(json_result)
}
