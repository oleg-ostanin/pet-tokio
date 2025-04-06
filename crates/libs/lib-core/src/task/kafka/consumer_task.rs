use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;
use tracing::{info, instrument};

use anyhow::Result;
use rdkafka::{ClientConfig, Message};
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::util::Timeout;
use lib_dto::order::OrderStored;
use crate::context::app_context::{AppConfig, ModelManager};
use crate::task::kafka::consumer_task::KafkaConsumerResponse::HealthOk;
use crate::task::main_task::{MainTaskRequest, TaskManager};

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
    pub(crate) async fn start(
        app_context: Arc<ModelManager>,
        app_config: AppConfig,
    ) -> Sender<KafkaConsumerRequest> {
        info!("Starting Kafka Consumer  Task");
        let main_tx = app_context.main_tx();

        let consumer = create(app_config).await;
        let task = {
            Self { consumer }
        };
        let (tx, rx) = tokio::sync::mpsc::channel(64);
        tokio::spawn(task.handle_kafka_consumer(app_context, rx));
        tx.clone()
    }

    #[instrument(skip_all)]
    pub async fn handle_kafka_consumer(
        self,
        app_context: Arc<ModelManager>,
        mut kafka_rx: Receiver<KafkaConsumerRequest>,
    ) -> Result<()> {
        info!("Starting kafka consumer task");
        self.consumer.subscribe(
            &["order-topic"]
        ).expect("Can't Subscribe");

        loop {
            match self.consumer.recv().await {
                Err(e) => println!("{:?}",e),
                Ok(message) => {
                    match message.payload_view::<str>() {
                        None => println!("None message"),
                        Some(Ok(msg)) => println!("Message Consumed : {}", msg),
                        Some(Err(e)) => println!("Error Parsing : {}",e)
                    }
                    self.consumer.commit_message(&message, CommitMode::Async).unwrap();
                }
            }
        }

        Ok(())
    }
}

async fn create(app_config: AppConfig) -> StreamConsumer {
    info!("Creating Kafka Consumer");

    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", app_config.kafka_url.as_str())
        .set("auto.offset.reset", "earliest")
        .set("group.id", "test-group")
        .set("socket.timeout.ms","4000");

    let consumer : StreamConsumer =
        config.create()
            .expect("Fail to create consumer");

    consumer
}


