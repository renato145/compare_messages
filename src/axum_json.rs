use crate::result::TestResult;
use anyhow::Result;
use fake::{faker::lorem::en::Words, Fake};
use log::debug;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonMessage {
    values: Vec<f64>,
    descriptions: Vec<String>,
}

pub async fn test_json_message(
    n_tests: usize,
    client: &reqwest::Client,
    num_elements: usize,
) -> Result<TestResult> {
    let msg = JsonMessage {
        values: fake::vec![f64; num_elements],
        descriptions: Words(num_elements..num_elements + 1).fake(),
    };

    let t0 = Instant::now();
    for _ in 0..n_tests {
        let msg = client
            .post("http://127.0.0.1:3000/json")
            .json(&msg)
            .send()
            .await?
            .json::<JsonMessage>()
            .await?;
        debug!("Client got: {:?}", msg);
    }
    let json_result = TestResult::new("Axum json", n_tests, num_elements, t0.elapsed());
    Ok(json_result)
}
