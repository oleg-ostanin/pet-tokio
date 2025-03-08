use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::select;
use tokio::sync::oneshot;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::sleep;
use tracing::{error, info};
use lib_dto::order::{OrderId, OrderItem, OrderItemExt, OrderStatus, OrderStored};
use lib_dto::order::OrderStatus::{Delivered, ReadyToDeliver};
use crate::bmc::book_info::BookBmc;
use crate::bmc::general::update_storage_and_order;
use crate::bmc::order::OrderBmc;
use crate::bmc::storage::StorageBmc;
use crate::bmc::storage::UpdateType::{Add, Remove};
use crate::context::app_context::ModelManager;
use crate::task::delivery::DeliveryResponse::HealthOk;
use crate::task::main::MainTaskRequest;

#[derive(Debug)]
pub(crate) enum DeliveryRequest {
    Health(oneshot::Sender<DeliveryResponse>),
    Deliver(OrderStored, oneshot::Sender<DeliveryResponse>),
}

#[derive(Debug)]
pub(crate) enum DeliveryResponse {
    HealthOk,
    Delivered,
    FailedToDeliver(OrderStored)
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

pub async fn handle_requests(
    main_tx: Sender<MainTaskRequest>,
    mut delivery_rx: Receiver<DeliveryRequest>,
) {
    let (o_tx, o_rx) = oneshot::channel();
    main_tx.send(MainTaskRequest::AppContext(o_tx)).await.expect("TODO: panic message");
    let app_context = o_rx.await.expect("TODO: panic message");

    while let Some(request) = delivery_rx.recv().await {
        match request {
            DeliveryRequest::Health(tx) => {
                tx.send(HealthOk).expect("TODO: panic message")
            }
            DeliveryRequest::Deliver(order, tx) => {
                tokio::spawn(handle_order(app_context.clone(), order, tx)).await.expect("TODO: panic message")
            }
        }
    }
}

pub async fn handle_order(
    app_context: Arc<ModelManager>,
    order: OrderStored,
    response_tx: oneshot::Sender<DeliveryResponse>
) {
    let order_id = order.order_id();
    info!("delivering order: {:?}", &order_id);
    select! {
        _ = update_with_retry(app_context, order.clone()) => {
            response_tx.send(DeliveryResponse::Delivered).expect("TODO: panic message");
        }
        _ = tokio::time::sleep(Duration::from_secs(3)) => {
            response_tx.send(DeliveryResponse::FailedToDeliver(order)).expect("TODO: panic message");
        }
    }

}

async fn update_with_retry(
    app_context: Arc<ModelManager>,
    order: OrderStored,
)  {
    while let Err(e) = update_storage_and_order(app_context.clone(), &order, Remove, Delivered).await {
        info!("delivery retrying update storage for order is: {:?}", &order);
        //sleep(Duration::from_millis(700)).await;
    }
}

pub async fn handle_delivery(
    app_context: Arc<ModelManager>,
    mut delivery_rx: Receiver<OrderStored>,
) {
    while let Some(order) = delivery_rx.recv().await {
        let order_id = order.order_id();
        info!("delivering order: {:?}", &order_id);

        while let Err(e) = update_storage_and_order(app_context.clone(), &order, Remove, Delivered).await {
            info!("delivery retrying update storage for order is: {:?}", &order);
            //sleep(Duration::from_millis(700)).await;
        }
    }
}