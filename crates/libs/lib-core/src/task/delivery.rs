use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;
use tracing::{info, instrument};

use lib_dto::order::OrderStatus::Delivered;
use lib_dto::order::OrderStored;

use crate::bmc::general::update_storage_and_order;
use crate::bmc::storage::UpdateType::Remove;
use crate::context::app_context::ModelManager;
use crate::select_cancel;
use crate::task::delivery::DeliveryResponse::HealthOk;
use crate::task::kafka::producer_task::KafkaProducerRequest;
use crate::task::main_task::{TaskManager};

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

pub(crate) struct DeliveryTask;

impl DeliveryTask {
    pub(crate) fn start(
        app_context: Arc<ModelManager>,
    ) -> Sender<DeliveryRequest> {
        let (tx, rx) = tokio::sync::mpsc::channel(64);

        let cancellation_token = app_context.cancellation_token();
        tokio::spawn(async move {
            select! {
                _ = handle_delivery_requests(app_context, rx) => {}
                _ = cancellation_token.cancelled() => {
                    info!("Cancelled by cancellation token.")
                }
            }
        });

        tx.clone()
    }
}

#[instrument(skip_all)]
pub async fn handle_delivery_requests(
    app_context: Arc<ModelManager>,
    mut delivery_rx: Receiver<DeliveryRequest>,
) -> Result<()> {
    info!("Starting handle delivery task");
    let main_tx = app_context.main_tx();
    let kafka_tx = TaskManager::kafka_producer_sender(main_tx.clone()).await?;
    info!("Got kafka_tx");

    let cancellation_token = app_context.cancellation_token();

    while let Some(request) = delivery_rx.recv().await {
        info!("Got delivery request: {:#?}", &request);

        let cancellation_token_cloned = cancellation_token.clone();
        let app_context_cloned = app_context.clone();

        match request {
            DeliveryRequest::Health(tx) => {
                tx.send(HealthOk).expect("TODO: panic message")
            }
            DeliveryRequest::Deliver(order, tx) => {
                info!("Sending kafka request: {:#?}", &order.order_id());
                kafka_tx.send(KafkaProducerRequest::ProduceOrder(order.clone())).await.expect("must be ok");
                select_cancel!(handle_order(app_context_cloned, order, tx), cancellation_token_cloned);
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