use anyhow::Result;
use axum::{routing::post, Json, Router};
use env_logger::Env;
use fake::{faker::lorem::en::Words, Fake};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    ops::Div,
    time::{Duration, Instant},
};

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    values: Vec<f64>,
    descriptions: Vec<String>,
}

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
        server().await;
    });

    let client = reqwest::Client::new();

    let tests = [10, 100, 1000];

    for n in tests {
        let result = test_message(n_tests, &client, n).await?;
        println!("{}", result);
    }

    Ok(())
}

struct TestResult {
    title: String,
    n_tests: usize,
    num_elements: usize,
    elapsed: Duration,
    elapsed_mean: Duration,
}

impl TestResult {
    fn new(
        title: impl Into<String>,
        n_tests: usize,
        num_elements: usize,
        elapsed: Duration,
    ) -> Self {
        let elapsed_mean = elapsed.div(n_tests.try_into().unwrap());
        Self {
            title: title.into(),
            n_tests,
            num_elements,
            elapsed,
            elapsed_mean,
        }
    }
}

impl std::fmt::Display for TestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} elements\t| elapsed ({} messages)={:?}\t| elapsed (mean)={:?}",
            self.title, self.num_elements, self.n_tests, self.elapsed, self.elapsed_mean
        )
    }
}

async fn test_message(
    n_tests: usize,
    client: &reqwest::Client,
    num_elements: usize,
) -> Result<TestResult> {
    let msg = Message {
        values: fake::vec![f64; num_elements],
        descriptions: Words(num_elements..num_elements + 1).fake(),
    };

    let json_result = {
        let t0 = Instant::now();
        for _ in 0..n_tests {
            let msg = client
                .post("http://127.0.0.1:3000/json")
                .json(&msg)
                .send()
                .await?
                .json::<Message>()
                .await?;
            debug!("Client got: {:?}", msg);
        }
        TestResult::new("JSON", n_tests, num_elements, t0.elapsed())
    };

    Ok(json_result)
}

async fn server() {
    let app = Router::new().route("/json", post(json_msg));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn json_msg(Json(message): Json<Message>) -> Json<Message> {
    debug!("Server got: {:?}", message);
    Json(message)
}
