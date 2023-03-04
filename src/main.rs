use axum::{response::Html, routing::get, Router, extract::State};
use lapin::{
    message::DeliveryResult,
    options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions},
    types::FieldTable,
    Connection, ConnectionProperties, ConsumerDelegate,
};
use std::{
    future::Future,
    net::SocketAddr,
    pin::Pin,
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug)]
struct Subscriber {
    count: Arc<Mutex<u32>>,
}

impl ConsumerDelegate for Subscriber {
    fn on_new_delivery(
        &self,
        delivery: DeliveryResult,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let count = self.count.clone();
        Box::pin(async move {
            if let Some(delivery) = delivery.unwrap() {
                let data = std::str::from_utf8(&delivery.data).unwrap();
                println!("{data}");

                {
                    let mut count = count.lock().unwrap();
                    *count += 1;
                    println!("{count}");
                }

                delivery.ack(BasicAckOptions::default()).await.unwrap();
            }
        })
    }
}

#[tokio::main]
async fn main() {
    let uri = "amqp://rabbitmq";
    let options = ConnectionProperties::default();

    let connection = Connection::connect(uri, options).await.unwrap();
    let channel = connection.create_channel().await.unwrap();

    let _queue = channel
        .queue_declare(
            "queue_test",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    let consumer = channel
        .basic_consume(
            "queue_test",
            "tag_foo",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    let count = Arc::new(Mutex::new(0));

    let consumer_count = count.clone();
    consumer.set_delegate(move |delivery: DeliveryResult| {
        let count = consumer_count.clone();
        async move {
            let delivery = match delivery {
                Ok(Some(delivery)) => delivery,
                Ok(None) => return,
                Err(error) => {
                    dbg!("Failed to consume queue message {}", error);
                    return;
                }
            };

            let data = std::str::from_utf8(&delivery.data).unwrap();
            println!("{data:?}");

            {
                let mut count = count.lock().unwrap();
                *count += 1;
            }

            delivery.ack(BasicAckOptions::default()).await.unwrap();
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