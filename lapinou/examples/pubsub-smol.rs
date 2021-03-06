use futures_util::future;
use lapin::{
    message::DeliveryResult, options::*, publisher_confirm::Confirmation, types::FieldTable,
    BasicProperties, Connection, ConnectionProperties, Result,
};
use lapinou::*;
use log::info;
use std::sync::Arc;

fn main() -> Result<()> {
    env_logger::init();

    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());

    // spawn a thread pool
    for _ in 0..5 {
        std::thread::spawn(|| smol::run(future::pending::<()>()));
    }

    smol::run(async {
        let conn = Connection::connect(&addr, ConnectionProperties::default().with_smol()).await?;

        info!("CONNECTED");

        let channel_a = conn.create_channel().await?;
        let channel_b = conn.create_channel().await?;

        let queue = channel_a
            .queue_declare(
                "hello",
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        info!("Declared queue {:?}", queue);

        let channel_b = Arc::new(channel_b);
        let consumer = channel_b
            .clone()
            .basic_consume(
                "hello",
                "my_consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        consumer.set_delegate(move |delivery: DeliveryResult| {
            let channel_b = channel_b.clone();
            async move {
                let delivery = delivery.expect("error caught in in consumer");
                if let Some(delivery) = delivery {
                    channel_b
                        .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                        .await
                        .expect("failed to ack");
                }
            }
        });

        let payload = b"Hello world!";

        loop {
            let confirm = channel_a
                .basic_publish(
                    "",
                    "hello",
                    BasicPublishOptions::default(),
                    payload.to_vec(),
                    BasicProperties::default(),
                )
                .await?
                .await?;
            assert_eq!(confirm, Confirmation::NotRequested);
        }
    })
}
