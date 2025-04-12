use anyhow::Result;
use log::error;
use rdkafka::{ClientConfig, Message};
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use tokio::sync::oneshot;
use tracing::{info, instrument};

use crate::context::app_context::{AppConfig, ModelManager};

#[derive(Debug)]
pub enum KafkaConsumerRequest {
    Health(oneshot::Sender<KafkaConsumerResponse>),
}

#[derive(Debug)]
pub enum KafkaConsumerResponse {
    HealthOk,
}

pub(crate) struct KafkaConsumerTask {
    consumer: StreamConsumer,
}

impl KafkaConsumerTask {
    #[instrument(skip_all)]
    pub(crate) async fn start(
        app_config: AppConfig,
    ) {
        info!("Starting Kafka Consumer Task");

        let consumer = create(app_config).await;
        let task = {
            Self { consumer }
        };
        tokio::spawn(task.handle_kafka_consumer());
    }

    #[instrument(skip_all)]
    pub async fn handle_kafka_consumer(
        self,
    ) -> Result<()> {
        info!("Handling kafka consumer task");
        self.consumer.subscribe(
            &["order-topic"]
        ).expect("Can't Subscribe");

        loop {
            match self.consumer.recv().await {
                Err(e) => error!("{:?}",e),
                Ok(message) => {
                    match message.payload_view::<str>() {
                        None => info!("None message"),
                        Some(Ok(msg)) => info!("Message Consumed : {}", msg),
                        Some(Err(e)) => error!("Error Parsing : {}",e)
                    }
                    self.consumer.commit_message(&message, CommitMode::Async).unwrap();
                }
            }
        }
    }
}

async fn create(app_config: AppConfig) -> StreamConsumer {
    info!("Creating Kafka Consumer");

    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", app_config.kafka_url.as_str())
        .set("auto.offset.reset", "earliest")
        .set("group.id", "test-group")
        .set("socket.timeout.ms","1000");

    let consumer : StreamConsumer =
        config.create()
            .expect("Fail to create consumer");

    consumer
}


