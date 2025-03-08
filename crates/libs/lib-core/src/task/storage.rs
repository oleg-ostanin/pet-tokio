use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use sqlx::Acquire;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;
use tokio::time::sleep;
use tracing::{info, instrument};

use lib_dto::order::{OrderId, OrderItem, OrderItemExt, OrderStatus, OrderStored};
use lib_dto::order::OrderStatus::{Delivered, ReadyToDeliver};
use crate::bmc::book_info::BookBmc;
use crate::bmc::general::update_storage_and_order;
use crate::bmc::order::OrderBmc;
use crate::bmc::storage::{StorageBmc, UpdateType};
use crate::bmc::storage::UpdateType::{Add, Remove};
use crate::context::app_context::ModelManager;
use crate::task::delivery::{DeliveryRequest, DeliveryResponse, DeliveryTask, handle_order};
use crate::task::main::MainTaskRequest;
use crate::task::storage::StorageResponse::HealthOk;

#[derive(Debug)]
pub(crate) enum StorageRequest {
    Health(oneshot::Sender<StorageResponse>),
    UpdateStorage(OrderStored, oneshot::Sender<StorageResponse>),
}

#[derive(Debug)]
pub(crate) enum StorageResponse {
    HealthOk,
    Updated,
    FailedToUpdate(OrderStored)
}

pub(crate) struct StorageTask {}

impl StorageTask {
    pub(crate) fn start(
        main_tx: Sender<MainTaskRequest>,
    ) -> Sender<StorageRequest> {
        let (tx, rx) = tokio::sync::mpsc::channel(64);
        tokio::spawn(handle_requests(main_tx, rx));
        tx.clone()
    }
}

pub async fn handle_requests(
    main_tx: Sender<MainTaskRequest>,
    mut storage_rx: Receiver<StorageRequest>,
) {
    let (o_tx, o_rx) = tokio::sync::oneshot::channel();
    main_tx.send(MainTaskRequest::AppContext(o_tx)).await.unwrap();
    let app_context = o_rx.blocking_recv().unwrap();

    while let Some(request) = storage_rx.recv().await {
        match request {
            StorageRequest::Health(tx) => {
                tx.send(HealthOk).unwrap()
            }
            StorageRequest::UpdateStorage(order, tx) => {
                tokio::spawn(handle_storage_new(app_context.clone(), order, tx)).await.unwrap()
            }
        }
    }
}

pub async fn handle_storage_new(
    app_context: Arc<ModelManager>,
    order: OrderStored,
    response_tx: oneshot::Sender<StorageResponse>
) {
    info!("updating storage for order: {:?}", &order);
    select! {
        _ = update_with_retry(app_context, order.clone()) => {
            response_tx.send(StorageResponse::Updated).unwrap();
        }
        _ = tokio::time::sleep(Duration::from_secs(3)) => {
            response_tx.send(StorageResponse::FailedToUpdate(order)).unwrap();
        }
    }
}

async fn update_with_retry(
    app_context: Arc<ModelManager>,
    order: OrderStored,
)  {
    while let Err(e) = update_storage_and_order(app_context.clone(), &order, Add, ReadyToDeliver).await {
        info!("delivery retrying update storage for order is: {:?}", &order);
        //sleep(Duration::from_millis(700)).await;
    }
}

pub async fn handle_storage(
    app_context: Arc<ModelManager>,
    mut order_item_rx: Receiver<OrderStored>,
    delivery_tx: Sender<OrderStored>,
) {
    while let Some(order) = order_item_rx.recv().await {
        info!("updating storage for order is: {:?}", &order);
        while let Err(e) = update_storage_and_order(app_context.clone(), &order, Add, ReadyToDeliver).await {
            info!("retrying update storage for order is: {:?}", &order);
            //sleep(Duration::from_millis(1000)).await;
        }
        delivery_tx.send(order).await.expect("ok")
    }
}

