use anyhow::Result;
use axum::{routing::post, Json, Router};
use fake::{faker::lorem::en::Words, Fake};
use futures::future::join_all;
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
    env_logger::init();
    let mut args = std::env::args();
    args.next();
    let n_tests: usize = args
        .next()
        .unwrap_or_else(|| "100".to_string())
        .parse()
        .unwrap();
    info!("Sending {} messages...", n_tests);

    tokio::spawn(async {
        server().await;
    });

    let client = reqwest::Client::new();

    let tests = [10, 100, 1000];

    let results = join_all(tests.map(|n| test_message(n_tests, &client, n)))
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    for (n, elapsed) in tests.iter().zip(results) {
        info!("----------------------------");
        info!("Number of elements: {}", n);
        info!("Elapsed: {:?}", elapsed);
        info!("Per msg: {:?}", elapsed.div(n_tests.try_into().unwrap()));
    }

    Ok(())
}

async fn test_message(
    n_tests: usize,
    client: &reqwest::Client,
    num_elements: usize,
) -> Result<Duration> {
    let msg = Message {
        values: fake::vec![f64; num_elements],
        descriptions: Words(num_elements..num_elements + 1).fake(),
    };

    let t0 = Instant::now();
    for _ in 0..n_tests {
        let msg = client
            .post("http://127.0.0.1:3000/")
            .json(&msg)
            .send()
            .await?
            .json::<Message>()
            .await?;
        debug!("Client got: {:?}", msg);
    }
    Ok(t0.elapsed())
}

async fn server() {
    let app = Router::new().route("/", post(root));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root(Json(message): Json<Message>) -> Json<Message> {
    debug!("Server got: {:?}", message);
    Json(message)
}
