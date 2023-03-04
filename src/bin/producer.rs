use lapin::{
    options::{BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties,
};

#[tokio::main]
async fn main() {
    let uri = "amqp://rabbitmq";
    let options = ConnectionProperties::default()
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);

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

    channel
        .basic_publish(
            "",
            "queue_test",
            BasicPublishOptions::default(),
            b"Hello world!",
            BasicProperties::default(),
        )
        .await
        .unwrap()
        .await
        .unwrap();
}
