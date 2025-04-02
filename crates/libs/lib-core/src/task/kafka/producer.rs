use std::time::Duration;
use rdkafka::{ClientConfig};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use tokio::sync::mpsc::Sender;
use tracing::info;
use crate::context::app_context::AppConfig;
use crate::task::main_task::{MainTaskRequest, TaskManager};

pub async fn create(app_config: AppConfig) -> FutureProducer {
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