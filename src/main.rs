use axum::{extract::State, response::Html, routing::get, Router};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio_stream::StreamExt;
use yamq::Broker;

#[tokio::main]
async fn main() {
    let broker = Broker::bind("127.0.0.1:1883").await;
    let mut subcriber = broker.subscription("#").await.unwrap();

    let count = Arc::new(Mutex::new(0));

    let subscriber_count = count.clone();
    tokio::spawn(async move {
        loop {
            let _message = subcriber.next().await;
            let mut count = subscriber_count.lock().unwrap();
            *count += 1;
        }
    });

    let app = Router::new()
        .route("/", get(handler))
        .route("/healthcheck", get(healthcheck))
        .route("/count", get(display_count))
        .with_state(count.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn healthcheck() {}

async fn display_count(State(count): State<Arc<Mutex<i32>>>) -> String {
    let count = count.lock().unwrap();
    format!("The count is {count}")
}
