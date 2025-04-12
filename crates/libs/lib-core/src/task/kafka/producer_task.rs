use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use log::error;
use rdkafka::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;
use tracing::{info, instrument};

use lib_dto::order::OrderStored;

use crate::context::app_context::{AppConfig, ModelManager};
use crate::task::kafka::producer_task::KafkaProducerResponse::HealthOk;

#[derive(Debug)]
pub enum KafkaProducerRequest {
    Health(oneshot::Sender<KafkaProducerResponse>),
    ProduceOrder(OrderStored),
}

#[derive(Debug)]
pub enum KafkaProducerResponse {
    HealthOk,
    Produced,
    FailedToProduce(OrderStored)
}

pub(crate) struct KafkaProducerTask {
    producer: FutureProducer,
}

impl KafkaProducerTask {
    pub(crate) async fn start(
        app_context: Arc<ModelManager>,
    ) -> Sender<KafkaProducerRequest> {
        info!("Starting kafka producer task");
        let app_config = app_context.app_config();
        let producer = create(app_config).await;
        info!("created producer");
        let task = {
            Self { producer }
        };
        let (tx, rx) = tokio::sync::mpsc::channel(64);
        tokio::spawn(task.handle_kafka_producer_requests(rx));
        tx.clone()
    }

    #[instrument(skip_all)]
    pub async fn handle_kafka_producer_requests(
        self,
        mut kafka_rx: Receiver<KafkaProducerRequest>,
    ) -> Result<()> {
        info!("Handling kafka requests");

        while let Some(request) = kafka_rx.recv().await {
            info!("Got Kafka request: {:#?}", &request);

            match request {
                KafkaProducerRequest::Health(tx) => {
                    tx.send(HealthOk).unwrap()
                }
                KafkaProducerRequest::ProduceOrder(order) => {
                    let producer = self.producer.clone();
                    tokio::spawn(produce(producer, order)).await.unwrap()
                }
            }
        }

        Ok(())
    }
}

#[instrument(skip_all)]
pub async fn create(app_config: &AppConfig) -> FutureProducer {
    info!("Creating kafka producer");

    info!("app_config: {:#?}", &app_config);

    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", app_config.kafka_url.as_str());
    //config.set("auto.create.topics.enable", "true");

    info!("config: {:#?}", &config);

    let producer : FutureProducer = config
        .create()
        .expect("Failure in creating producer");

    producer
}

#[instrument(skip_all)]
pub async fn produce(
    producer: FutureProducer,
    order: OrderStored, ) {
    info!("producing order: {:#?}", &order);
    let res = serde_json::to_string(&order).unwrap();
    let payload = res.as_str();
    let record = FutureRecord::to("order-topic")
        .payload(payload)
        .key("Test-Key");

    let status_delivery = producer
        .send(record, Timeout::After(Duration::from_secs(2)))
        .await;

    match status_delivery {
        Ok(report) => info!("Message Sent {:?}",report),
        Err(e) => error!("Error producing.. {:?}",e)
    }
}



