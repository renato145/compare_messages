use crate::{JsonMessage, TestResult};
use anyhow::Result;
use avro_rs::from_value;
use avro_rs::Reader;
use avro_rs::Schema;
use avro_rs::Writer;
use fake::{faker::lorem::en::Words, Fake};
use log::debug;
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

pub async fn test_avro_message(
    n_tests: usize,
    client: &reqwest::Client,
    num_elements: usize,
) -> Result<TestResult> {
    let msg = JsonMessage {
        values: fake::vec![f64; num_elements],
        descriptions: Words(num_elements..num_elements + 1).fake(),
    };
    let schema = Schema::parse_str(RAW_SCHEMA).unwrap();

    let t0 = Instant::now();
    for _ in 0..n_tests {
        let mut writer = Writer::new(&schema, Vec::new());
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
