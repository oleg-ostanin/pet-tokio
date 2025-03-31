use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::oneshot;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{info, instrument};
use lib_dto::order::OrderStored;
use lib_dto::order::OrderStatus::Delivered;
use crate::bmc::general::update_storage_and_order;
use crate::bmc::storage::UpdateType::Remove;
use crate::context::app_context::ModelManager;
use crate::task::delivery::DeliveryResponse::HealthOk;
use crate::task::main_task::{MainTaskRequest, TaskManager};
use anyhow::Result;
use crate::task::kafka::producer_task::KafkaProducerRequest;

#[derive(Debug)]
pub enum DeliveryRequest {
    Health(oneshot::Sender<DeliveryResponse>),
    Deliver(OrderStored, oneshot::Sender<DeliveryResponse>),
}

#[derive(Debug)]
pub enum DeliveryResponse {
    HealthOk,
    Delivered,

    // contains order_id
    FailedToDeliver(i64)
}

pub(crate) struct DeliveryTask {
    app_context: Arc<ModelManager>,
    tx: Sender<DeliveryRequest>,
    rx: Receiver<DeliveryRequest>,
}

impl DeliveryTask {
    pub(crate) fn start(
        main_tx: Sender<MainTaskRequest>,
    ) -> Sender<DeliveryRequest> {
        let (tx, rx) = tokio::sync::mpsc::channel(64);
        tokio::spawn(handle_requests(main_tx, rx));
        tx.clone()
    }
}

#[instrument(skip_all)]
pub async fn handle_requests(
    main_tx: Sender<MainTaskRequest>,
    mut delivery_rx: Receiver<DeliveryRequest>,
) -> Result<()> {
    info!("Starting handle delivery task");
    let app_context = TaskManager::app_context(main_tx.clone()).await?;
    //let kafka_tx = TaskManager::kafka_producer_sender(main_tx.clone()).await?;

    while let Some(request) = delivery_rx.recv().await {
        info!("Got delivery request: {:#?}", &request);
        match request {
            DeliveryRequest::Health(tx) => {
                tx.send(HealthOk).expect("TODO: panic message")
            }
            DeliveryRequest::Deliver(order, tx) => {
                //kafka_tx.send(KafkaProducerRequest::ProduceOrder(order.clone())).await.expect("must be ok");
                tokio::spawn(handle_order(app_context.clone(), order, tx)).await.expect("TODO: panic message")
            }
        }
    }

    Ok(())
}

#[instrument(skip_all)]
pub async fn handle_order(
    app_context: Arc<ModelManager>,
    order: OrderStored,
    response_tx: oneshot::Sender<DeliveryResponse>
) {
    let order_id = order.order_id();
    info!("delivering order: {:#?}", &order_id);
    select! {
        // todo think about cancellation safety here
        _ = update_with_retry(app_context, &order) => {
            response_tx.send(DeliveryResponse::Delivered).expect("TODO: panic message");
        }
        _ = tokio::time::sleep(Duration::from_secs(3)) => {
            response_tx.send(DeliveryResponse::FailedToDeliver(order_id)).expect("TODO: panic message");
        }
    }
}

#[instrument(skip_all)]
async fn update_with_retry(
    app_context: Arc<ModelManager>,
    order: &OrderStored,
)  {
    while let Err(e) = update_storage_and_order(app_context.clone(), &order, Remove, Delivered).await {
        info!("delivery retrying update storage for order is: {:#?} because of {:#?}", &order, e);
    }
}