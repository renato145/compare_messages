use crate::{JsonMessage, TestResult};
use anyhow::Result;
use avro_rs::{from_value, Reader, Schema, Writer};
use fake::{faker::lorem::en::Words, Fake};
use log::debug;
use once_cell::sync::Lazy;
use std::time::Instant;

pub const RAW_SCHEMA: &str = r#"
    {
        "type": "record",
        "name": "test",
        "fields": [
            {"name": "values", "type": "array", "items": "double"},
            {"name": "descriptions", "type": "array", "items": "string"}
        ]
    }
"#;

pub static SCHEMA: Lazy<Schema> = Lazy::new(|| Schema::parse_str(RAW_SCHEMA).unwrap());

pub async fn test_avro_message(
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
        let mut writer = Writer::new(&SCHEMA, Vec::new());
        writer.append_ser(&msg)?;
        let encoded = writer.into_inner()?;

        let msg = client
            .post("http://127.0.0.1:3000/avro")
            .body(encoded)
            .send()
            .await?
            .bytes()
            .await?;

        let value = Reader::new(&msg[..])?.into_iter().next().unwrap()?;
        let decoded = from_value::<JsonMessage>(&value)?;
        debug!("Client got: {:?}", decoded);
    }
    let json_result = TestResult::new("Axum avro", n_tests, num_elements, t0.elapsed());
    Ok(json_result)
}
