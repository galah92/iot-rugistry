use axum::{extract::State, response::Html, routing::get, Router};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use yamq::Broker;

#[tokio::main]
async fn main() {
    let broker = Broker::bind("127.0.0.1:1883").await;
    let mut subcriber = broker.subscription("#").await.unwrap();

    let shared_state = Arc::new(RwLock::new(AppState::default()));

    let subscriber_state = shared_state.clone();
    tokio::spawn(async move {
        loop {
            let msg = subcriber.next().await.unwrap().unwrap();
            let topic = msg.topic.as_ref().to_string();
            let payload = String::from_utf8(msg.payload.to_vec()).unwrap();
            subscriber_state.write().await.db.insert(topic, payload);
        }
    });

    let app = Router::new()
        .route("/", get(handler))
        .route("/healthcheck", get(healthcheck))
        .route("/state", get(display_state))
        .with_state(shared_state.clone());

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

async fn display_state(State(state): State<SharedState>) -> String {
    let db = &state.read().await.db;
    let data = format!("{:?}", db);
    data
}

type SharedState = Arc<RwLock<AppState>>;

#[derive(Default)]
struct AppState {
    db: HashMap<String, String>,
}
