use std::time::Duration;
use rdkafka::{ClientConfig};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use tokio::sync::mpsc::Sender;
use tracing::info;
use crate::task::main_task::{MainTaskRequest, TaskManager};

pub async fn create(main_tx: Sender<MainTaskRequest>) -> FutureProducer {
    info!("Creating kafka producer");

    let app_context = TaskManager::app_context(main_tx).await.expect("must be ok");
    let app_config = app_context.app_config();

    info!("app_config: {:#?}", &app_config);

    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", app_config.kafka_url.as_str());
    config.set("auto.create.topics.enable", "true");

    info!("config: {:#?}", &config);

    let producer : FutureProducer = config
        .create()
        .expect("Failure in creating producer");

    producer
}