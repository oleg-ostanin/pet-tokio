use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;
use tracing::{info, instrument};

use anyhow::Result;
use rdkafka::consumer::StreamConsumer;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use lib_dto::order::OrderStored;
use crate::context::app_context::ModelManager;
use crate::task::kafka::producer_task::KafkaResponse::HealthOk;
use crate::task::kafka::producer::create;
use crate::task::main_task::{MainTaskRequest, TaskManager};

#[derive(Debug)]
pub enum KafkaRequest {
    Health(oneshot::Sender<KafkaResponse>),
    ProduceOrder(OrderStored, oneshot::Sender<KafkaResponse>),
}

#[derive(Debug)]
pub enum KafkaResponse {
    HealthOk,
    Produced,
    FailedToProduce(OrderStored)
}

pub(crate) struct KafkaProducerTask {
    producer: FutureProducer,
}

impl KafkaProducerTask {
    pub(crate) fn start(
        main_tx: Sender<MainTaskRequest>,
    ) -> Sender<KafkaRequest> {
        let producer = create();
        let task = {
            Self { producer }
        };
        let (tx, rx) = tokio::sync::mpsc::channel(64);
        tokio::spawn(task.handle_requests(main_tx, rx));
        tx.clone()
    }

    #[instrument(skip_all)]
    pub async fn handle_requests(
        self,
        main_tx: Sender<MainTaskRequest>,
        mut kafka_rx: Receiver<KafkaRequest>,
    ) -> Result<()> {
        info!("Starting kafka task");
        let app_context = TaskManager::app_context(main_tx.clone()).await?;

        while let Some(request) = kafka_rx.recv().await {
            info!("Got Kafka request: {:#?}", &request);

            match request {
                KafkaRequest::Health(tx) => {
                    tx.send(HealthOk).unwrap()
                }
                KafkaRequest::ProduceOrder(order, tx) => {
                    let producer = self.producer.clone();
                    tokio::spawn(produce(producer, order, tx)).await.unwrap()
                }
            }
        }

        Ok(())
    }
}

#[instrument(skip_all)]
pub async fn produce(
    producer: FutureProducer,
    order: OrderStored,
    response_tx: oneshot::Sender<KafkaResponse>
) {
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
        Ok(report) => println!("Message Sent {:?}",report),
        Err(e) => println!("Error producing.. {:?}",e)
    }
}



