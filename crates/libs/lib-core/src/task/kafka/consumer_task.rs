use std::ops::Deref;
use std::sync::Arc;
use anyhow::Result;
use log::error;
use rdkafka::{ClientConfig, Message};
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use tokio::select;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
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
        app_context: Arc<ModelManager>,
    ) {
        info!("Starting Kafka Consumer Task");

        let app_config: AppConfig = app_context.app_config().clone();
        let consumer = create(app_config, "main-group").await;
        let task = {
            Self { consumer }
        };

        let cancellation_token = app_context.cancellation_token();
        let jh = tokio::spawn(async move {
            select! {
                _ = task.handle_kafka_consumer() => {}
                _ = cancellation_token.cancelled() => {
                    info!("Cancelled by cancellation token.")
                }
            }
        }).await;

        match jh {
            Ok(_) => {
                info!("task finished ok")
            }
            Err(e) => {
                info!("task finished with error: {:?}", e)
            }
        }
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

pub async fn create(app_config: AppConfig, group_id: &str) -> StreamConsumer {
    info!("Creating Kafka Consumer");

    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", app_config.kafka_url.as_str())
        .set("auto.offset.reset", "earliest")
        .set("group.id", group_id)
        .set("socket.timeout.ms","1000");

    let consumer : StreamConsumer =
        config.create()
            .expect("Fail to create consumer");

    consumer
}


